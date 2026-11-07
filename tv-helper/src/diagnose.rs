//! Read-only diagnosis: what this TV reports and which actions could be offered. It only
//! reads files and checks that paths exist; it never writes, runs commands or needs root.
use crate::{
    home::{self, Home},
    system::{self, System},
};
use serde_json::{Value, json};

const RELEASE: &str = "/etc/starfish-release";
/// Major webOS versions the Home integration is written for. Not proof that a TV works.
const HOME_VERSIONS: [u32; 4] = [5, 6, 10, 11];

pub fn report(system: &impl System) -> Value {
    let root = system.uid() == 0;
    let raw = system::read(system, RELEASE, 4096)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    let major = raw.as_deref().and_then(home::release_major);
    let known = major.is_some_and(|major| HOME_VERSIONS.contains(&major));
    let luna_send = system::exists(system, "/usr/bin/luna-send");
    let mount = system::exists(system, "/bin/mount") && system::exists(system, "/bin/umount");
    let init_d = system::exists(system, "/var/lib/webosbrew/init.d");
    let upstream = Home::new(system, home::HOME)
        .and_then(|home| home.reject_upstream_integration())
        .err();

    let mut reasons = vec![];
    if !root {
        reasons.push("Run the TV helper as root".to_owned());
    }
    match major {
        None => reasons.push("Unknown webOS version".to_owned()),
        Some(major) if !known => reasons.push(format!("Unsupported webOS version {major}")),
        Some(_) => {}
    }
    if !luna_send {
        reasons.push("luna-send is missing".into());
    }
    if !mount {
        reasons.push("mount or umount is missing".into());
    }
    if let Some(reason) = &upstream {
        reasons.push(reason.clone());
    }
    json!({
        "returnValue": true,
        "readOnly": true,
        "root": root,
        "release": {"raw": raw, "major": major, "known": known},
        "capabilities": {
            "lunaSend": luna_send,
            "mount": mount,
            "initD": init_d,
            "upstreamIntegration": upstream.is_some(),
        },
        // "possible" only means nothing found blocks it; nothing is verified on this TV.
        "actions": [{
            "id": "home-replacement",
            "state": if reasons.is_empty() { "possible" } else { "blocked" },
            "verified": false,
            "reasons": reasons,
        }],
    })
}
