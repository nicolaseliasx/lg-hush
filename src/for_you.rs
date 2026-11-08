//! "Para Você" data: Cinemeta catalogs (Stremio's public catalog) reduced to cards.
//! No account or key; titles and genres come in English, genres are translated here.
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

/// Cards shown at most; also the number of cached images.
pub const COUNT: usize = 12;
const FRESH_SECONDS: i64 = 6 * 3600;

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
    pub genres: Vec<String>,
    /// Minutes (per episode for series).
    pub runtime: Option<u32>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Cache {
    /// Unix seconds of the fetch.
    pub fetched: i64,
    pub items: Vec<Item>,
}
impl Cache {
    pub fn fresh(&self, now: i64) -> bool {
        (self.fetched..self.fetched + FRESH_SECONDS).contains(&now)
    }
}

/// IMDb ids only: they become image file names and launch parameters.
pub fn valid_id(id: &str) -> bool {
    id.strip_prefix("tt").is_some_and(|digits| {
        (1..=10).contains(&digits.len()) && digits.bytes().all(|b| b.is_ascii_digit())
    })
}

pub fn parse_catalog(catalog: &Value, kind: Kind, limit: usize) -> Vec<Item> {
    catalog["metas"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|meta| {
            let id = meta["id"].as_str().filter(|id| valid_id(id))?;
            let title = meta["name"]
                .as_str()
                .map(str::trim)
                .filter(|t| !t.is_empty())?;
            let genres = meta["genres"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .take(3)
                .map(str::to_owned)
                .collect();
            let runtime = meta["runtime"]
                .as_str()
                .and_then(|r| r.split_whitespace().next())
                .and_then(|m| m.parse::<u32>().ok())
                .filter(|m| (1..=1000).contains(m));
            Some(Item {
                id: id.into(),
                kind,
                title: title.chars().take(120).collect(),
                genres,
                runtime,
            })
        })
        .take(limit)
        .collect()
}

/// Movie, series, movie, … until `count`; whichever list is longer fills the rest.
pub fn interleave(movies: Vec<Item>, series: Vec<Item>, count: usize) -> Vec<Item> {
    let mut seen = HashSet::new();
    let (mut movies, mut series) = (movies.into_iter(), series.into_iter());
    let mut items = vec![];
    while items.len() < count {
        let next = if items.len() % 2 == 0 {
            movies.next().or_else(|| series.next())
        } else {
            series.next().or_else(|| movies.next())
        };
        let Some(item) = next else { break };
        if seen.insert(item.id.clone()) {
            items.push(item);
        }
    }
    items
}

fn genre(name: &str, language: Language) -> &str {
    if language != Language::Pt {
        return name;
    }
    match name {
        "Action" => "Ação",
        "Adventure" => "Aventura",
        "Animation" => "Animação",
        "Biography" => "Biografia",
        "Comedy" => "Comédia",
        "Documentary" => "Documentário",
        "Family" => "Família",
        "Fantasy" => "Fantasia",
        "History" => "História",
        "Horror" => "Terror",
        "Music" => "Música",
        "Mystery" => "Mistério",
        "Sci-Fi" => "Ficção científica",
        "Sport" => "Esporte",
        "Thriller" => "Suspense",
        "War" => "Guerra",
        "Western" => "Faroeste",
        "News" => "Notícias",
        "Reality-TV" => "Reality",
        _ => name,
    }
}

/// "Terror · 1h 49min": first genre and runtime, each only when known.
pub fn caption(item: &Item, language: Language) -> String {
    let minute = if language == Language::En { "m" } else { "min" };
    let runtime = item.runtime.map(|m| match (m / 60, m % 60) {
        (0, m) => format!("{m}{minute}"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}{minute}"),
    });
    item.genres
        .first()
        .map(|g| genre(g, language).to_owned())
        .into_iter()
        .chain(runtime)
        .collect::<Vec<_>>()
        .join(" · ")
}

/// The Home row: cached cards, which images are on disk, and its own horizontal scroll.
#[derive(Default)]
pub struct Row {
    pub items: Vec<Item>,
    pub ready: HashSet<String>,
    pub fetched: i64,
    /// A refresh was started this session.
    pub requested: bool,
    pub scroll: f32,
    pub images: PathBuf,
    pub warm: Warm,
}
impl Row {
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
