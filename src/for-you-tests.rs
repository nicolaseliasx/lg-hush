use super::*;
use crate::i18n::Language;
use serde_json::json;

fn movie(id: &str, name: &str, genres: &[&str], runtime: &str) -> serde_json::Value {
    json!({"id": id, "type": "movie", "name": name, "genres": genres, "runtime": runtime})
}

#[test]
fn cinemeta_catalogs_become_items_and_skip_invalid_entries() {
    let catalog = json!({"metas": [
        movie("tt37287335", "Obsession", &["Horror", "Thriller"], "109 min"),
        movie("tt0111161", "The Shawshank Redemption", &["Drama"], ""),
        {"id": "kitsu:1", "name": "Not IMDb", "genres": []},
        {"id": "tt1", "name": "  ", "genres": []},
        {"id": "tt2"},
        "garbage",
    ]});
    let items = parse_catalog(&catalog, Kind::Movie, 12);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, "tt37287335");
    assert_eq!(items[0].title, "Obsession");
    assert_eq!(items[0].genres, vec!["Horror", "Thriller"]);
    assert_eq!(items[0].runtime, Some(109));
    assert_eq!(items[1].runtime, None);
    assert!(parse_catalog(&json!({"metas": "x"}), Kind::Series, 12).is_empty());
}

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
fn movies_and_series_alternate_up_to_the_limit() {
    let item = |id: &str, kind| Item {
        id: id.into(),
        kind,
        title: id.into(),
        genres: vec![],
        runtime: None,
    };
    let movies = vec![
        item("tt1", Kind::Movie),
        item("tt2", Kind::Movie),
        item("tt3", Kind::Movie),
    ];
    let series = vec![item("tt9", Kind::Series)];
    let ids: Vec<_> = interleave(movies, series, 3)
        .into_iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(ids, ["tt1", "tt9", "tt2"]);
}

#[test]
fn captions_show_the_first_genre_and_runtime_in_the_owners_language() {
    let mut item = Item {
        id: "tt1".into(),
        kind: Kind::Movie,
        title: "Obsession".into(),
        genres: vec!["Horror".into(), "Thriller".into()],
        runtime: Some(109),
    };
    assert_eq!(caption(&item, Language::Pt), "Terror · 1h 49min");
    assert_eq!(caption(&item, Language::En), "Horror · 1h 49m");
    item.runtime = Some(45);
    item.genres = vec!["Sci-Fi".into()];
    assert_eq!(caption(&item, Language::Pt), "Ficção científica · 45min");
    item.runtime = None;
    item.genres = vec!["Unknown-Genre".into()];
    assert_eq!(caption(&item, Language::Pt), "Unknown-Genre");
    item.genres.clear();
    assert_eq!(caption(&item, Language::Pt), "");
}

#[test]
fn the_cache_is_fresh_for_six_hours() {
    let cache = Cache {
        fetched: 1_000_000,
        items: vec![],
    };
    assert!(cache.fresh(1_000_000 + 6 * 3600 - 1));
    assert!(!cache.fresh(1_000_000 + 6 * 3600));
    assert!(
        !cache.fresh(1_000_000 - 120),
        "a clock set back does not pin old data"
    );
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
