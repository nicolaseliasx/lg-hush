use super::{controls, scene};
use crate::{
    for_you::{Item, Kind, Resume},
    model::{Preferences, demo_inventory},
    state::{Direction, Effect, Page, State, Target},
};

fn item(id: &str) -> Item {
    Item {
        id: id.into(),
        kind: Kind::Movie,
        title: format!("Title {id}"),
        resume: Resume {
            progress: 0.,
            remaining: Some(100),
            season: None,
            episode: None,
        },
    }
}

/// The fork's quiet Home with three titles in progress.
fn home() -> State {
    let mut state = State::new(Preferences::default());
    state.set_inventory(demo_inventory(), None);
    state.for_you.items = vec![item("tt1"), item("tt2"), item("tt3")];
    state
}

fn press(state: &mut State, direction: Direction) -> Vec<Effect> {
    let controls = controls(state);
    state.direction(direction, &controls)
}

#[test]
fn for_you_is_on_by_default_but_off_in_the_legacy_look_and_for_old_saved_preferences_on() {
    assert!(Preferences::default().show_for_you);
    assert!(!Preferences::legacy_defaults().show_for_you);
    let saved: Preferences = serde_json::from_str(r#"{"favorites":["netflix"]}"#).unwrap();
    assert!(saved.show_for_you);
}

#[test]
fn the_row_sits_between_the_tabs_and_favorites() {
    let mut state = home();
    state.focus = Target::Nav(0);
    press(&mut state, Direction::Down);
    assert_eq!(state.focus, Target::ForYou("tt1".into()));
    press(&mut state, Direction::Right);
    assert_eq!(state.focus, Target::ForYou("tt2".into()));
    press(&mut state, Direction::Down);
    assert!(matches!(state.focus, Target::App(_)), "{:?}", state.focus);
    press(&mut state, Direction::Up);
    assert!(
        matches!(state.focus, Target::ForYou(_)),
        "{:?}",
        state.focus
    );
    press(&mut state, Direction::Up);
    assert_eq!(state.focus, Target::Nav(0));
}

#[test]
fn ok_on_a_card_opens_its_title() {
    let mut state = home();
    state.focus = Target::ForYou("tt2".into());
    assert_eq!(state.activate(), vec![Effect::OpenTitle(item("tt2"))]);
}

#[test]
fn portrait_cards_sit_above_favorites_which_move_down_only_while_the_row_shows() {
    use crate::state::{FOR_YOU_HEIGHT, FOR_YOU_WIDTH};
    let state = home();
    let controls = controls(&state);
    let card = controls
        .iter()
        .find(|c| c.target == Target::ForYou("tt1".into()))
        .expect("card control");
    assert!(
        (FOR_YOU_WIDTH / FOR_YOU_HEIGHT - 2. / 3.).abs() < 0.01,
        "posters are 2:3"
    );
    assert_eq!((card.rect.w, card.rect.h), (FOR_YOU_WIDTH, FOR_YOU_HEIGHT));
    assert!(card.rect.y > 108.8);
    let tile_y = controls_y(&state);
    assert!(
        tile_y > card.rect.y + card.rect.h + 60.,
        "Favorites below the cards and the info line"
    );
    let mut off = home();
    off.preferences.show_for_you = false;
    let mut empty = home();
    empty.for_you.items.clear();
    assert_eq!(controls_y(&off), 544.8);
    assert_eq!(controls_y(&empty), 544.8);
}

#[test]
fn only_the_focused_title_is_named_in_one_line_under_the_row() {
    let texts = |state: &State| -> Vec<String> {
        scene(state)
            .draw
            .iter()
            .filter_map(|d| match d {
                crate::renderer::Draw::Text(t, ..) => Some(t.clone()),
                _ => None,
            })
            .collect()
    };
    let mut state = home();
    // With posters on disk, cards carry no text; without one the title is the placeholder.
    state.for_you.ready = ["tt1", "tt2", "tt3"].map(String::from).into();
    state.focus = Target::ForYou("tt2".into());
    let shown = texts(&state);
    assert!(
        shown
            .iter()
            .any(|t| t.starts_with("Title tt2") && t.contains("1h 40")),
        "{shown:?}"
    );
    assert!(!shown.iter().any(|t| t.contains("Title tt1")));
    state.focus = Target::Nav(0);
    assert!(!texts(&state).iter().any(|t| t.contains("Title tt")));
}

fn controls_y(state: &State) -> f32 {
    controls(state)
        .iter()
        .filter(|c| matches!(c.target, Target::App(_)))
        .map(|c| c.rect.y)
        .fold(f32::INFINITY, f32::min)
}

#[test]
fn the_row_is_hidden_without_titles_or_when_switched_off() {
    let mut empty = home();
    empty.for_you.items.clear();
    let mut off = home();
    off.preferences.show_for_you = false;
    for mut state in [empty, off] {
        assert!(
            !controls(&state)
                .iter()
                .any(|c| matches!(c.target, Target::ForYou(_)))
        );
        state.focus = Target::Nav(0);
        press(&mut state, Direction::Down);
        assert!(matches!(state.focus, Target::App(_)), "{:?}", state.focus);
    }
}

#[test]
fn the_widgets_setting_switches_the_row() {
    let mut state = home();
    state.page = Page::Settings;
    state.section = crate::state::WIDGETS_SECTION;
    state.focus = Target::Widget(5);
    assert!(
        scene(&state)
            .controls
            .iter()
            .any(|c| c.target == Target::Widget(5))
    );
    assert_eq!(state.activate(), vec![Effect::Save]);
    assert!(!state.preferences.show_for_you);
}

fn texts(state: &State) -> Vec<String> {
    scene(state)
        .draw
        .iter()
        .filter_map(|d| match d {
            crate::renderer::Draw::Text(t, ..) => Some(t.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_row_is_continue_watching_with_where_each_title_stopped() {
    let mut state = home();
    let language = state.preferences.language;
    state.for_you.items = vec![Item {
        kind: Kind::Series,
        resume: Resume {
            progress: 0.4,
            remaining: Some(52),
            season: Some(1),
            episode: Some(1),
        },
        ..item("tt9")
    }];
    let shown = texts(&state);
    assert!(
        shown.iter().any(|t| t == language.t("continueWatching")),
        "{shown:?}"
    );
    assert!(
        !shown.iter().any(|t| t == "For You"),
        "the catalog row is gone"
    );
    state.focus = Target::ForYou("tt9".into());
    let expected = format!(
        "Title tt9  ·  {}",
        crate::for_you::caption(&state.for_you.items[0], language)
    );
    assert!(texts(&state).contains(&expected), "{expected}");
    assert_eq!(
        state.activate(),
        vec![Effect::OpenTitle(state.for_you.items[0].clone())],
        "OK opens it in Stremio"
    );
}

#[test]
fn a_title_in_progress_shows_how_far_it_was_watched() {
    // The bar is the only drawing that differs between the two states.
    let shapes = |state: &State| {
        scene(state)
            .draw
            .iter()
            .filter(|d| matches!(d, crate::renderer::Draw::Shape(..)))
            .count()
    };
    let mut state = home();
    state.for_you.ready = ["tt1", "tt2", "tt3"].map(String::from).into();
    let before = shapes(&state);
    state.for_you.items[0].resume.progress = 0.4;
    assert_eq!(shapes(&state), before + 2, "a track and its filled part");
}

#[test]
fn widgets_settings_offer_only_continue_watching_and_inputs() {
    let mut state = home();
    state.page = Page::Settings;
    state.section = crate::state::WIDGETS_SECTION;
    let language = state.preferences.language;
    let rows = scene(&state)
        .controls
        .iter()
        .filter(|c| matches!(c.target, Target::Widget(_)))
        .count();
    assert_eq!(rows, 2);
    let shown = texts(&state);
    for key in ["continueWatching", "inputs"] {
        assert!(
            shown.iter().any(|t| t == language.t(key)),
            "{key}: {shown:?}"
        );
    }
    for label in ["Weather", "Greeting", "Date", "Shortcuts"] {
        assert!(!shown.iter().any(|t| t == label), "{label}: {shown:?}");
    }
}

#[test]
fn old_preferences_with_retired_widgets_keep_a_quiet_home() {
    // Saved before the greeting, date, weather and shortcuts were removed.
    let saved: Preferences = serde_json::from_str(
        r#"{"show_weather":true,"show_greeting":true,"show_date":true,"greeting_name":"Ana",
            "custom_tiles":[{"id":"t1","title":"Lights","url":"http://192.0.2.1/hook"}]}"#,
    )
    .unwrap();
    let mut state = State::new(saved);
    state.set_inventory(demo_inventory(), None);
    state.for_you.items = home().for_you.items;
    assert!(
        controls(&state)
            .iter()
            .any(|c| matches!(c.target, Target::ForYou(_))),
        "Continue Watching still shows"
    );
    let shown = texts(&state);
    assert!(
        !shown
            .iter()
            .any(|t| t.contains("Lights") || t.contains("Ana")),
        "{shown:?}"
    );
}

/// Wide rounded fills under the first favorite: the dock and nothing else of its kind.
fn docks(state: &State) -> Vec<(crate::state::Rect, [f32; 4])> {
    let scene = scene(state);
    let Some(first) = scene
        .controls
        .iter()
        .find(|c| matches!(c.target, Target::App(_)))
        .map(|c| c.rect)
    else {
        return vec![];
    };
    scene
        .draw
        .iter()
        .filter_map(|d| match d {
            crate::renderer::Draw::Shape(rect, radius, stroke, color)
                if *stroke == 0.
                    && *radius >= 24.
                    && rect.w > 1500.
                    && rect.contains(first.x + 1., first.y + 1.) =>
            {
                Some((*rect, *color))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn favorites_sit_on_a_slightly_lighter_dock() {
    let mut state = home();
    let tiles: Vec<_> = controls(&state)
        .into_iter()
        .filter(|c| matches!(c.target, Target::App(_)))
        .map(|c| c.rect)
        .collect();
    let found = docks(&state);
    assert_eq!(found.len(), 1, "one dock behind Favorites");
    let (dock, color) = found[0];
    assert!(
        color[3] > 0. && color[3] <= 0.1,
        "a slightly lighter black, not a bright panel: {color:?}"
    );
    for tile in tiles.iter().filter(|t| t.x + t.w <= 1748.) {
        assert!(dock.x < tile.x && dock.x + dock.w > tile.x + tile.w);
        assert!(dock.y < tile.y && dock.y + dock.h > tile.y + tile.h);
    }
    // The dock stays put while the favorites scroll inside it.
    state.home_scroll = 281.6;
    assert!(scene(&state).draw.iter().any(|d| matches!(d,
        crate::renderer::Draw::Shape(rect, 36., 0., _) if rect.x == dock.x && rect.w == dock.w)));
}

#[test]
fn qr_codes_carry_no_luma_logo() {
    // A full-size dark QR: every module set, so a gap left for a logo would be visible.
    let mut state = home();
    state.page = Page::Settings;
    state.overlay = Some(crate::state::Overlay::WallpaperUpload);
    state.wallpaper_upload = Some(Ok(crate::companion::Session {
        url: "http://192.0.2.1:8080/abc".into(),
        qr_size: 29,
        qr: vec![true; 29 * 29],
    }));
    let drawn = scene(&state);
    assert!(
        !drawn.draw.iter().any(|d| matches!(d,
            crate::renderer::Draw::TintedImage(path, ..) if path.contains("luma-mark"))),
        "no logo in the middle of the code"
    );
    // Each of the 29 rows is one merged run of modules, the middle rows included.
    let rows = drawn
        .draw
        .iter()
        .filter(|d| matches!(d, crate::renderer::Draw::Shape(_, 0., 0., color) if color[0] > 0.9))
        .count();
    assert_eq!(rows, 29);
}

#[test]
fn favorites_are_two_percent_larger_than_before_and_the_apps_grid_is_unchanged() {
    let state = home();
    let tiles: Vec<_> = controls(&state)
        .into_iter()
        .filter(|c| matches!(c.target, Target::App(_)))
        .map(|c| c.rect)
        .collect();
    let (w, h) = (256. * 1.02, 154.4 * 1.02);
    assert!(
        (tiles[0].w - w).abs() < 0.01 && (tiles[0].h - h).abs() < 0.01,
        "{:?}",
        tiles[0]
    );
    assert!(
        (tiles[1].x - tiles[0].x - tiles[0].w - 25.6).abs() < 0.01,
        "same gap"
    );
    let full = tiles
        .iter()
        .filter(|t| t.x + t.w <= crate::state::HOME_STRIP_RIGHT + 0.1)
        .count();
    assert_eq!(full, 6, "six whole favorites, the seventh cut off");
    let mut apps = home();
    apps.preferences.show_app_names = false;
    apps.open(1);
    let grid = controls(&apps)
        .into_iter()
        .find(|c| matches!(c.target, Target::App(_)))
        .unwrap()
        .rect;
    assert_eq!((grid.w, grid.h), (270., 154.4), "Apps keeps its size");
}
