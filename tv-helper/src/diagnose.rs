//! Read-only diagnosis: what this TV reports and which actions could be offered. It only
//! reads files and checks that paths exist; it never writes, runs commands or needs root.
use crate::{
    home::{self, Home},
    system::{self, System},
};
use serde_json::{Value, json};

const RELEASE: &str = "/etc/starfish-release";

pub fn report(system: &impl System) -> Value {
    let root = system.uid() == 0;
    let raw = system::read(system, RELEASE, 4096)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    let major = raw.as_deref().and_then(home::release_major);
    let luna_send = system::exists(system, "/usr/bin/luna-send");
    let mount = system::exists(system, "/bin/mount") && system::exists(system, "/bin/umount");
    let init_d = system::exists(system, "/var/lib/webosbrew/init.d");
    let upstream = Home::new(system, home::HOME)
        .and_then(|home| home.reject_upstream_integration())
        .err();
    // The read-only checks `enable` makes, in its order; the first refusal is reported.
    let method = Home::new(system, home::HOME).and_then(|home| {
        home.preflight()?;
        let Some(release) = raw.as_deref().filter(|_| major.is_some()) else {
            return Ok(None);
        };
        let stock = home.stock(home::load(system)?.as_ref())?;
        let method = home::method(release, &stock, home::HOME)?;
        home.conflicts(&stock)?;
        Ok(Some(method))
    });

    let mut reasons = vec![];
    if !root {
        reasons.push("Run the TV helper as root".to_owned());
    }
    if major.is_none() {
        reasons.push("Unknown webOS version".to_owned());
    }
    if !luna_send {
        reasons.push("luna-send is missing".into());
    }
    if !mount {
        reasons.push("mount or umount is missing".into());
    }
    if let Err(reason) = &method {
        reasons.push(reason.clone());
    }
    json!({
        "returnValue": true,
        "readOnly": true,
        "root": root,
        "release": {"raw": raw, "major": major},
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
            "method": method.ok().flatten(),
            "verified": false,
            "reasons": reasons,
        }],
    })
}
