use super::{controls, scene};
use crate::{
    for_you::{Item, Kind},
    model::{Preferences, demo_inventory},
    state::{Direction, Effect, Page, State, Target},
};

fn item(id: &str) -> Item {
    Item {
        id: id.into(),
        kind: Kind::Movie,
        title: format!("Title {id}"),
        genres: vec!["Drama".into()],
        runtime: Some(100),
    }
}

/// The fork's quiet Home (no greeting, date or weather) with three cached titles.
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
            .any(|t| t.starts_with("Title tt2") && t.contains("Drama")),
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
fn the_row_is_hidden_without_titles_or_when_a_home_widget_uses_that_space() {
    let mut empty = home();
    empty.for_you.items.clear();
    let mut greeting = home();
    greeting.preferences.show_greeting = true;
    let mut off = home();
    off.preferences.show_for_you = false;
    for mut state in [empty, greeting, off] {
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
