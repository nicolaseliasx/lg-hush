use super::*;
use crate::i18n::Language;
use serde_json::json;

#[test]
fn ids_are_imdb_only_so_they_are_safe_as_file_names() {
    assert!(valid_id("tt0111161"));
    for id in [
        "",
        "tt",
        "0111161",
        "tt0111161/..",
        "tt../x",
        "tt123456789012",
    ] {
        assert!(!valid_id(id), "{id}");
    }
}

#[test]
fn stremio_cards_is_kept_warm_after_startup_and_rechecked_every_thirty_seconds() {
    // Back in Stremio Cards closes it; the next card must not wait for a cold start.
    use std::time::{Duration, Instant};
    let start = Instant::now();
    let at = |s: u64| start + Duration::from_secs(s);
    let mut warm = Warm::default();
    assert!(
        !warm.due(at(0), true, true),
        "not before the first Home frame settles"
    );
    warm.first_frame(at(0));
    assert!(!warm.due(at(9), true, true));
    assert!(
        !warm.due(at(10), false, true),
        "only while Home is on screen"
    );
    assert!(!warm.due(at(10), true, false), "only when installed");
    assert!(warm.due(at(10), true, true));
    assert!(!warm.due(at(39), true, true));
    assert!(warm.due(at(40), true, true));
}

#[allow(clippy::too_many_arguments)]
fn saved(
    id: &str,
    kind: &str,
    name: &str,
    offset: u64,
    duration: u64,
    episode: (u32, u32),
    removed: bool,
    last: &str,
) -> serde_json::Value {
    json!({
        "_id": id, "type": kind, "name": name, "removed": removed, "temp": false,
        "state": {"timeOffset": offset, "duration": duration, "season": episode.0,
                  "episode": episode.1, "lastWatched": last}
    })
}

#[test]
fn the_stremio_library_becomes_continue_watching_by_stremios_own_rule() {
    // Stremio lists an item while it has progress and is either kept in the library or was
    // only watched (temp); most recent first.
    let mut watched_only = saved(
        "tt2",
        "series",
        "Dexter: Ressurreição",
        18_027,
        3_149_502,
        (1, 1),
        true,
        "2026-11-07T23:09:04.955Z",
    );
    watched_only["temp"] = json!(true);
    let library = json!({"result": [
        saved("tt6", "series", "Older", 60_000, 0, (2, 5), false, "2026-10-01T00:00:00.000Z"),
        watched_only,
        saved("tt1", "movie", "Harry Potter", 7_723_275, 8_504_208, (0, 0), false,
              "2026-11-08T09:48:08.453Z"),
        saved("tt3", "movie", "Not started", 0, 9_000_000, (0, 0), false,
              "2026-11-08T10:00:00.000Z"),
        saved("tt4", "movie", "Removed", 5_000, 10_000, (0, 0), true, "2026-11-08T10:00:00.000Z"),
        saved("tt5", "other", "A channel", 5_000, 10_000, (0, 0), false,
              "2026-11-08T10:00:00.000Z"),
        saved("kitsu:1", "series", "Not IMDb", 5_000, 10_000, (1, 1), false,
              "2026-11-08T10:00:00.000Z"),
        "garbage",
    ]});
    let items = parse_library(&library, 8);
    let ids: Vec<_> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, ["tt1", "tt2", "tt6"]);
    assert_eq!(items[0].kind, Kind::Movie);
    assert_eq!(items[0].title, "Harry Potter");
    let movie = &items[0].resume;
    assert!((movie.progress - 0.908).abs() < 0.001, "{}", movie.progress);
    assert_eq!(
        (movie.remaining, movie.season, movie.episode),
        (Some(13), None, None)
    );
    let series = &items[1].resume;
    assert_eq!(items[1].kind, Kind::Series);
    assert_eq!(
        (series.remaining, series.season, series.episode),
        (Some(52), Some(1), Some(1))
    );
    let unknown = &items[2].resume;
    assert_eq!(
        (unknown.progress, unknown.remaining),
        (0., None),
        "no duration, no bar or time left"
    );
    assert_eq!(parse_library(&library, 1).len(), 1);
    assert!(parse_library(&json!({"error": {"message": "x"}}), 8).is_empty());
}

fn resumed(id: &str, kind: Kind, remaining: Option<u32>, episode: Option<(u32, u32)>) -> Item {
    Item {
        id: id.into(),
        kind,
        title: id.into(),
        resume: Resume {
            progress: 0.5,
            remaining,
            season: episode.map(|e| e.0),
            episode: episode.map(|e| e.1),
        },
    }
}

#[test]
fn continue_watching_captions_name_the_episode_and_the_time_left() {
    let episode = resumed("tt1", Kind::Series, Some(52), Some((1, 1)));
    assert_eq!(caption(&episode, Language::Pt), "T1 E1 · faltam 52min");
    assert_eq!(caption(&episode, Language::En), "S1 E1 · 52m left");
    let movie = resumed("tt2", Kind::Movie, Some(133), None);
    assert_eq!(caption(&movie, Language::Pt), "faltam 2h 13min");
    assert_eq!(
        caption(&resumed("tt3", Kind::Movie, None, None), Language::Pt),
        ""
    );
}

#[test]
fn continue_watching_is_checked_at_most_once_a_minute() {
    use std::time::{Duration, Instant};
    let start = Instant::now();
    let mut row = Row::default();
    assert!(row.resume_due(start));
    assert!(!row.resume_due(start + Duration::from_secs(59)));
    assert!(row.resume_due(start + Duration::from_secs(60)));
}

#[test]
fn the_saved_stremio_key_is_read_only_when_it_is_plausible() {
    assert_eq!(
        auth_key(br#"{"authKey":"abc/DEF+123="}"#).as_deref(),
        Some("abc/DEF+123=")
    );
    for invalid in [
        &br#"{"authKey":""}"#[..],
        br#"{"authKey":"a b"}"#,
        br#"{"authKey":7}"#,
        br#"{}"#,
        b"not json",
    ] {
        assert_eq!(
            auth_key(invalid),
            None,
            "{}",
            String::from_utf8_lossy(invalid)
        );
    }
    let long = format!(r#"{{"authKey":"{}"}}"#, "a".repeat(513));
    assert_eq!(auth_key(long.as_bytes()), None);
}
