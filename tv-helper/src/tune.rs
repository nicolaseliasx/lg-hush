//! Performance debloat while Luma is Home (webOS 6.5, LG 55UP751C0SF): stop preloading apps
//! the owner does not use, keep LG's suggestion, ad and viewing-tracking services from
//! running, and block LG ad hosts. Nothing is uninstalled or written to system partitions:
//! services get a do-nothing stub bind-mounted over their binary and /etc/hosts gets a RAM
//! copy, both gone at the next reboot until the boot hook applies them again. The preload
//! list goes to configd (persistent, also covers standby wake) and to the preload manager
//! (immediate). LG's values are saved before the first change and restored from that copy.
use crate::{
    Result, accepted,
    system::{self, System},
};
use serde_json::{Value, json};
use std::path::Path;

const STATE: &str = "/var/lib/ncls-luma-tune";
const CONFIG: &str = "/var/lib/ncls-luma-tune/state.json";
const STUB: &str = "/var/lib/ncls-luma-tune/stub";
const HOSTS: &str = "/var/run/ncls-luma-hosts";
const PRELOAD_KEY: &str = "com.webos.service.preloadmanager.supportAppList";
const MARK: &str = "# ncls-luma-tune";
/// The owner uses no antenna, HDMI device, voice, AirPlay, HomeKit or ThinQ (2026-11-07).
/// YouTube and Home keep LG's setting.
pub const PRELOADS_OFF: [&str; 15] = [
    "netflix",
    "com.webos.app.browser",
    "amazon",
    "globoplaywebos",
    "com.webos.app.voice",
    "com.webos.app.livetv",
    "com.webos.app.livemenu",
    "com.webos.app.hdmi1",
    "com.webos.app.hdmi2",
    "com.webos.app.hdmi3",
    "com.webos.app.hdmi4",
    "com.webos.app.miracast",
    "com.webos.app.miracast-overlay",
    "google.assistant",
    "airplay",
];
pub const AD_HOSTS: [&str; 3] = [
    "eic.ads.lgtvcommon.com",
    "nl.ad.lgsmartad.com",
    "nl.info.lgsmartad.com",
];
/// (binary, how it is started) for suggestions, ads, viewing tracking, telemetry and the
/// Apple integrations the owner does not use.
pub const SERVICES: [(&str, Start); 18] = [
    ("/usr/sbin/contentminer", Start::Systemd("contentminer")),
    ("/usr/sbin/nudge", Start::Systemd("nudge")),
    ("/usr/sbin/remotelogger", Start::Systemd("remotelogger")),
    ("/usr/bin/sportsalarm", Start::Upstart("sportsalarm")),
    // On webOS 6.5 the bus starts it from /usr/sbin (seen on the TV), not the upstart job.
    ("/usr/sbin/sportsalarm", Start::Bus),
    ("/usr/sbin/acr2", Start::Bus),
    ("/usr/sbin/adoverlay-service", Start::Bus),
    ("/usr/sbin/admanager", Start::Bus),
    ("/usr/sbin/rdxd", Start::Systemd("rdxd")),
    ("/usr/sbin/uploadd", Start::Systemd("uploadd")),
    ("/usr/sbin/homekit", Start::Bus),
    ("/usr/sbin/airplay", Start::Bus),
    // Voice assistants (started by jailer on demand) and the broadcast configuration download;
    // the owner uses neither voice nor an antenna.
    ("/usr/sbin/amazon-alexa-adapter", Start::Bus),
    ("/usr/sbin/amazon-alexa-vsk", Start::Bus),
    ("/usr/sbin/google-assistant-adapter", Start::Bus),
    ("/usr/sbin/lg.thinqai.adapter", Start::Bus),
    ("/usr/sbin/dangbei-adapter", Start::Bus),
    (
        "/usr/sbin/broadcast-conf-downloader",
        Start::Systemd("broadcast-conf-downloader"),
    ),
];
/// JavaScript services run by node (run-js-service): an empty directory is mounted over the
/// service, and the running copy (process title = service name) is ended.
pub const JS_SERVICES: [&str; 1] = ["/usr/palm/services/com.webos.service.homeconnect"];
const EMPTY: &str = "/var/lib/ncls-luma-tune/empty";
#[derive(Clone, Copy)]
pub enum Start {
    Systemd(&'static str),
    Upstart(&'static str),
    /// Started on demand by the bus: end a running copy; the stub answers the next
    /// activation.
    Bus,
}

fn luna(system: &impl System, uri: &str, payload: Value) -> Result<Value> {
    let (service, method) = uri.rsplit_once('/').ok_or("Invalid Luna URI")?;
    accepted(system::luna(system, service, method, &payload)?)
}
/// Our stub mounted over the binary, also an older stub file replaced since (the mount then
/// keeps the deleted inode, so comparing files is not enough).
fn stubbed(system: &impl System, binary: &str) -> bool {
    ours(system, binary, "ncls-luma-tune/stub")
}
fn masked(system: &impl System, dir: &str) -> bool {
    ours(system, dir, "ncls-luma-tune/empty")
}
fn ours(system: &impl System, target: &str, source: &str) -> bool {
    system::read(system, "/proc/self/mountinfo", 1 << 20).is_ok_and(|info| {
        String::from_utf8_lossy(&info).lines().any(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            fields.len() > 4
                && fields[4] == target
                && fields[3].trim_end_matches("//deleted").ends_with(source)
        })
    })
}
fn name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
/// Ends processes whose title (first argument) is the JS service name.
fn end_titled(system: &impl System, title: &str) -> Result<()> {
    let entries = std::fs::read_dir(system.path(Path::new("/proc"))).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|pid| *pid > 1 && *pid <= i32::MAX as u32)
        else {
            continue;
        };
        let Ok(cmdline) = system::read(system, format!("/proc/{pid}/cmdline"), 64 * 1024) else {
            continue;
        };
        let first = cmdline.split(|b| *b == 0).next().unwrap_or_default();
        if String::from_utf8_lossy(first).trim() == title {
            system.terminate(pid)?;
        }
    }
    Ok(())
}
/// Ends processes still running the binary's original file: SIGTERM, then SIGKILL after 3 s
/// (homekit ignores SIGTERM). The stub is already mounted, so nothing restarts the original.
fn end(system: &impl System, binary: &str) -> Result<()> {
    let pids = running(system, binary)?;
    for pid in &pids {
        system.terminate(*pid)?;
    }
    for _ in 0..3 {
        if pids
            .iter()
            .all(|pid| !system::exists(system, format!("/proc/{pid}")))
        {
            return Ok(());
        }
        system.pause();
    }
    for pid in pids {
        if system::exists(system, format!("/proc/{pid}")) {
            system.force(pid)?;
        }
    }
    Ok(())
}
fn running(system: &impl System, binary: &str) -> Result<Vec<u32>> {
    let mut pids = vec![];
    let target = system.path(Path::new(binary));
    let entries = std::fs::read_dir(system.path(Path::new("/proc"))).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|pid| *pid > 1 && *pid <= i32::MAX as u32)
        else {
            continue;
        };
        let exe = std::fs::read_link(entry.path().join("exe")).ok();
        if exe.is_some_and(|p| {
            Path::new(p.to_str().unwrap_or("").trim_end_matches(" (deleted)")) == target
        }) {
            pids.push(pid);
        }
    }
    Ok(pids)
}
fn hosts_mounted(system: &impl System) -> bool {
    system::same_file(system, HOSTS, "/etc/hosts")
}
fn unmount(system: &impl System, path: &str) -> Result<()> {
    if let Err(error) = system::command(system, "/bin/umount", &[path]) {
        eprintln!("Luma tune: {error}; detaching mount");
        system::command(system, "/bin/umount", &["-l", path])?;
    }
    Ok(())
}
fn lg_preloads(system: &impl System) -> Result<Value> {
    let reply = luna(
        system,
        "com.webos.service.config/getConfigs",
        json!({"configNames":[PRELOAD_KEY]}),
    )?;
    let list = &reply["configs"][PRELOAD_KEY];
    if !list.is_array() {
        return Err("This TV has no preload list in configd".into());
    }
    Ok(list.clone())
}
fn set_preloads(system: &impl System, list: &Value, live: &[(String, bool)]) -> Result<()> {
    luna(
        system,
        "com.webos.service.config/setConfigs",
        json!({"configs":{PRELOAD_KEY: list}}),
    )?;
    for (id, enabled) in live {
        let payload = json!({"application":{"id":id,"isEnabled":enabled}});
        // At cold boot the preload manager stays silent well past the hook (36-60 s on
        // webOS 6.5). configd already holds the list it reads at its next start, so the
        // immediate policy is only a warning when it does not answer.
        let mut attempt = 0;
        loop {
            match luna(
                system,
                "com.webos.service.preloadmanager/setPreloadPolicy",
                payload.clone(),
            ) {
                Ok(_) => break,
                Err(e) if attempt >= 10 => {
                    eprintln!("Luma tune: preload policy for {id} not applied now: {e}");
                    return Ok(());
                }
                Err(_) => {
                    attempt += 1;
                    system.pause();
                }
            }
        }
    }
    Ok(())
}
fn without(lg: &Value, off: &[&str]) -> Value {
    Value::Array(
        lg.as_array()
            .into_iter()
            .flatten()
            .map(|entry| {
                let mut entry = entry.clone();
                if entry["id"].as_str().is_some_and(|id| off.contains(&id)) {
                    entry["isEnabled"] = json!(false);
                }
                entry
            })
            .collect(),
    )
}
fn home_enabled(system: &impl System) -> Result<bool> {
    Ok(crate::home::load(system)?.is_some_and(|c| c.enabled))
}

pub fn apply(system: &impl System) -> Result<Value> {
    system::root(system)?;
    if !home_enabled(system)? {
        return Err("Enable Luma as Home first: LG Home relies on some of these services".into());
    }
    system::mkdir(system, STATE)?;
    // The first copy of LG's values is kept across re-applications.
    let mut state = if system::exists(system, CONFIG) {
        system::json(system, CONFIG)?
    } else {
        json!({"enabled":false,"preloads":lg_preloads(system)?})
    };
    state["enabled"] = json!(true);
    system::write_json(system, CONFIG, &state)?;
    let mut failed = vec![];

    let live: Vec<_> = PRELOADS_OFF
        .iter()
        .map(|id| (id.to_string(), false))
        .collect();
    if let Err(e) = set_preloads(system, &without(&state["preloads"], &PRELOADS_OFF), &live) {
        failed.push(format!("preloads: {e}"));
    }

    // Rewriting would give the stub a new inode under mounts that already use it.
    if system::read(system, STUB, 64).ok().as_deref() != Some(b"#!/bin/sh\nexit 0\n") {
        system::write(system, STUB, b"#!/bin/sh\nexit 0\n", 0o755)?;
    }
    for (binary, start) in SERVICES {
        if !system::exists(system, binary) {
            continue;
        }
        // Stub first, so a restart or bus activation finds nothing to run.
        if !stubbed(system, binary)
            && let Err(e) = system::command(system, "/bin/mount", &["--bind", STUB, binary])
        {
            failed.push(format!("{binary}: {e}"));
            continue;
        }
        let stopped = match start {
            // nudge ignores SIGTERM past the helper's deadline; systemd finishes the stop.
            Start::Systemd(unit) => system::command(
                system,
                "/bin/systemctl",
                &["stop", "--no-block", &format!("{unit}.service")],
            ),
            Start::Upstart(job) => {
                // initctl fails when the job is already stopped; that is the goal.
                let _ = system::command(system, "/usr/bin/initctl", &["stop", job]);
                Ok(())
            }
            Start::Bus => end(system, binary),
        };
        if let Err(e) = stopped {
            failed.push(format!("{binary}: {e}"));
        }
    }

    system::mkdir(system, EMPTY)?;
    for dir in JS_SERVICES {
        if !system::exists(system, dir) {
            continue;
        }
        if !masked(system, dir)
            && let Err(e) = system::command(system, "/bin/mount", &["--bind", EMPTY, dir])
        {
            failed.push(format!("{dir}: {e}"));
            continue;
        }
        if let Err(e) = end_titled(system, &name(dir)) {
            failed.push(format!("{dir}: {e}"));
        }
    }

    if !hosts_mounted(system) {
        let mut hosts =
            String::from_utf8_lossy(&system::read(system, "/etc/hosts", 1 << 20)?).into_owned();
        if !hosts.ends_with('\n') {
            hosts.push('\n');
        }
        for host in AD_HOSTS {
            hosts.push_str(&format!("127.0.0.1 {host} {MARK}\n::1 {host} {MARK}\n"));
        }
        let result = system::write(system, HOSTS, hosts.as_bytes(), 0o644)
            .and_then(|()| system::command(system, "/bin/mount", &["--bind", HOSTS, "/etc/hosts"]));
        if let Err(e) = result {
            failed.push(format!("/etc/hosts: {e}"));
        }
    }
    if failed.is_empty() {
        Ok(status(system))
    } else {
        Err(format!(
            "Partly applied (restore with: tune restore): {}",
            failed.join("; ")
        ))
    }
}

pub fn restore(system: &impl System) -> Result<Value> {
    system::root(system)?;
    let mut failed = vec![];
    if hosts_mounted(system)
        && let Err(e) = unmount(system, "/etc/hosts")
    {
        failed.push(format!("/etc/hosts: {e}"));
    }
    for (binary, start) in SERVICES {
        if !stubbed(system, binary) {
            continue;
        }
        if let Err(e) = unmount(system, binary) {
            failed.push(format!("{binary}: {e}"));
            continue;
        }
        let started = match start {
            Start::Systemd(unit) => system::command(
                system,
                "/bin/systemctl",
                &["start", &format!("{unit}.service")],
            ),
            Start::Upstart(job) => system::command(system, "/usr/bin/initctl", &["start", job]),
            Start::Bus => Ok(()),
        };
        if let Err(e) = started {
            failed.push(format!("{binary}: {e}"));
        }
    }
    for dir in JS_SERVICES {
        if masked(system, dir)
            && let Err(e) = unmount(system, dir)
        {
            failed.push(format!("{dir}: {e}"));
        }
    }
    if system::exists(system, CONFIG) {
        let state = system::json(system, CONFIG)?;
        let lg = &state["preloads"];
        let live: Vec<_> = PRELOADS_OFF
            .iter()
            .filter_map(|id| {
                lg.as_array()?
                    .iter()
                    .find(|e| e["id"] == *id)
                    .map(|e| (id.to_string(), e["isEnabled"] == true))
            })
            .collect();
        match set_preloads(system, lg, &live) {
            // LG's values are back: the saved copy is no longer needed.
            Ok(()) if failed.is_empty() => system::remove(system, CONFIG)?,
            Ok(()) => {}
            Err(e) => failed.push(format!("preloads: {e}")),
        }
    }
    if failed.is_empty() {
        Ok(status(system))
    } else {
        Err(format!("Partly restored: {}", failed.join("; ")))
    }
}

pub fn status(system: &impl System) -> Value {
    let enabled = system::json(system, CONFIG).is_ok_and(|s| s["enabled"] == true);
    let mut services: serde_json::Map<_, _> = SERVICES
        .iter()
        .filter(|(binary, _)| system::exists(system, binary))
        .map(|(binary, _)| (name(binary), json!(stubbed(system, binary))))
        .collect();
    for dir in JS_SERVICES {
        if system::exists(system, dir) {
            services.insert(name(dir), json!(masked(system, dir)));
        }
    }
    json!({
        "returnValue": true,
        "enabled": enabled,
        "preloadsOff": PRELOADS_OFF,
        "servicesStubbed": services,
        "adHostsBlocked": hosts_mounted(system),
    })
}

/// Boot hook: re-apply after a cold boot when the owner enabled it.
pub fn boot(system: &impl System) -> Result<Value> {
    if system::json(system, CONFIG).is_ok_and(|s| s["enabled"] == true) {
        apply(system)
    } else {
        Ok(json!({"returnValue":true,"enabled":false}))
    }
}
