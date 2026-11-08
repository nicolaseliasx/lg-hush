//! "Continue Watching": titles in progress in the owner's Stremio library, read with a session
//! key saved on the TV (never in this repository).
use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// The Stremio build that opens a title from launch parameters; kept preloaded.
pub const STREMIO_CARDS: &str = "cc.ncls.stremiocards";
/// The official app (ignores launch parameters on webOS 6.5).
pub const STREMIO: &str = "io.strem.tv";

/// Titles in progress shown at most; also the number of cached posters.
pub const COUNT: usize = 8;
const RESUME_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Movie,
    Series,
}
impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Series => "series",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Item {
    pub id: String,
    pub kind: Kind,
    pub title: String,
    pub resume: Resume,
}

/// Where the owner stopped: the share watched and, when known, the time left and episode.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Resume {
    /// 0 to 1; 0 when the duration is unknown.
    pub progress: f32,
    /// Minutes left.
    pub remaining: Option<u32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
}

/// IMDb ids only: they become image file names and launch parameters.
pub fn valid_id(id: &str) -> bool {
    id.strip_prefix("tt").is_some_and(|digits| {
        (1..=10).contains(&digits.len()) && digits.bytes().all(|b| b.is_ascii_digit())
    })
}

/// Stremio's "Continue Watching" from a `datastoreGet` answer for `libraryItem`: movies and
/// series with progress that are kept in the library or were only watched (`temp`), the most
/// recently watched first. The same rule as Stremio's own row.
pub fn parse_library(response: &Value, limit: usize) -> Vec<Item> {
    let mut entries: Vec<(&str, Item)> = response["result"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let id = entry["_id"].as_str().filter(|id| valid_id(id))?;
            let kind = match entry["type"].as_str()? {
                "movie" => Kind::Movie,
                "series" => Kind::Series,
                _ => return None,
            };
            let removed = entry["removed"].as_bool().unwrap_or(false);
            let temp = entry["temp"].as_bool().unwrap_or(false);
            let state = &entry["state"];
            let offset = state["timeOffset"].as_u64().unwrap_or(0);
            if offset == 0 || (removed && !temp) {
                return None;
            }
            let title = entry["name"]
                .as_str()
                .map(str::trim)
                .filter(|t| !t.is_empty())?;
            let duration = state["duration"].as_u64().unwrap_or(0);
            let (progress, remaining) = if duration > 0 {
                let left = duration.saturating_sub(offset);
                (
                    (offset as f64 / duration as f64).min(1.) as f32,
                    Some(((left + 30_000) / 60_000) as u32),
                )
            } else {
                (0., None)
            };
            let number = |key: &str| {
                state[key]
                    .as_u64()
                    .filter(|n| kind == Kind::Series && (1..=10_000).contains(n))
                    .map(|n| n as u32)
            };
            let last = state["lastWatched"].as_str().unwrap_or("");
            Some((
                last,
                Item {
                    id: id.into(),
                    kind,
                    title: title.chars().take(120).collect(),
                    resume: Resume {
                        progress,
                        remaining,
                        season: number("season"),
                        episode: number("episode"),
                    },
                },
            ))
        })
        .collect();
    // ISO 8601 timestamps in UTC sort as text.
    entries.sort_by(|a, b| b.0.cmp(a.0));
    entries
        .into_iter()
        .map(|(_, item)| item)
        .take(limit)
        .collect()
}

/// The session key from the file `scripts/stremio-login.sh` writes: `{"authKey": "…"}`.
pub fn auth_key(bytes: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(bytes).ok()?;
    value["authKey"]
        .as_str()
        .filter(|key| (1..=512).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_graphic()))
        .map(str::to_owned)
}

fn duration(minutes: u32, language: Language) -> String {
    let minute = if language == Language::En { "m" } else { "min" };
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m}{minute}"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}{minute}"),
    }
}

/// Where the title stopped, each part only when known: "T1 E1 · faltam 52min".
pub fn caption(item: &Item, language: Language) -> String {
    let resume = &item.resume;
    let episode = resume.season.zip(resume.episode).map(|(s, e)| {
        language.format(
            "resumeEpisode",
            &[("season", &s.to_string()), ("episode", &e.to_string())],
        )
    });
    let left = resume
        .remaining
        .filter(|m| *m > 0)
        .map(|m| language.format("resumeLeft", &[("time", &duration(m, language))]));
    episode
        .into_iter()
        .chain(left)
        .collect::<Vec<_>>()
        .join(" · ")
}

/// The Home row: titles in progress, which posters are on disk, and its own horizontal scroll.
#[derive(Default)]
pub struct Row {
    pub items: Vec<Item>,
    resume_checked: Option<std::time::Instant>,
    pub ready: HashSet<String>,
    pub scroll: f32,
    pub images: PathBuf,
    pub warm: Warm,
}
impl Row {
    /// The library is read again at most once a minute while Home is on screen, so coming
    /// back from Stremio shows where the owner stopped.
    pub fn resume_due(&mut self, now: std::time::Instant) -> bool {
        if self
            .resume_checked
            .is_some_and(|last| now < last + RESUME_INTERVAL)
        {
            return false;
        }
        self.resume_checked = Some(now);
        true
    }
    /// The image file, only once the service has saved it (a failed load is never retried).
    pub fn image(&self, id: &str) -> Option<String> {
        self.ready
            .contains(id)
            .then(|| image_path(&self.images, id).to_string_lossy().into_owned())
    }
}

/// When to preload Stremio Cards so a card opens without the cold start (~2 s instead of
/// 10-15 s on webOS 6.5): 10 s after the first Home frame, then at most every 30 s while Home
/// is on screen, since Back inside Stremio Cards closes it (and the system may close it too).
#[derive(Default)]
pub struct Warm {
    first_frame: Option<std::time::Instant>,
    last: Option<std::time::Instant>,
}
impl Warm {
    pub fn first_frame(&mut self, now: std::time::Instant) {
        self.first_frame.get_or_insert(now);
    }
    pub fn due(&mut self, now: std::time::Instant, home: bool, installed: bool) -> bool {
        use std::time::Duration;
        let settled = self
            .first_frame
            .is_some_and(|first| now >= first + Duration::from_secs(10));
        let rested = self
            .last
            .is_none_or(|last| now >= last + Duration::from_secs(30));
        if home && installed && settled && rested {
            self.last = Some(now);
            return true;
        }
        false
    }
}

pub fn image_path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.jpg"))
}

#[cfg(test)]
#[path = "for-you-tests.rs"]
mod tests;
