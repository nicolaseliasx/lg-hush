//! Bundled translations, including Simplified and Traditional Chinese.
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};
use time::OffsetDateTime;

type Messages = BTreeMap<String, BTreeMap<String, String>>;
fn messages() -> &'static Messages {
    static CATALOG: OnceLock<Messages> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../assets/messages.json"))
            .expect("Bundled translations must be valid")
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    De,
    Zh,
    #[serde(rename = "zh-tw", alias = "zh-TW", alias = "zh-Hant")]
    ZhTw,
    Es,
    Pt,
    It,
    Ru,
    Pl,
    Fr,
    Vi,
    Nl,
    Sv,
}
impl Language {
    pub const ALL: [Self; 13] = [
        Self::De,
        Self::En,
        Self::Es,
        Self::Fr,
        Self::It,
        Self::Nl,
        Self::Pl,
        Self::Pt,
        Self::Sv,
        Self::Vi,
        Self::Ru,
        Self::Zh,
        Self::ZhTw,
    ];
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::De => "de",
            Self::Zh => "zh",
            Self::ZhTw => "zh-tw",
            Self::Es => "es",
            Self::Pt => "pt",
            Self::It => "it",
            Self::Ru => "ru",
            Self::Pl => "pl",
            Self::Fr => "fr",
            Self::Vi => "vi",
            Self::Nl => "nl",
            Self::Sv => "sv",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::De => "Deutsch",
            Self::Zh => "简体中文",
            Self::ZhTw => "繁體中文",
            Self::Es => "Español",
            Self::Pt => "Português (Brasil)",
            Self::It => "Italiano",
            Self::Ru => "Русский",
            Self::Pl => "Polski",
            Self::Fr => "Français",
            Self::Vi => "Tiếng Việt",
            Self::Nl => "Nederlands",
            Self::Sv => "Svenska",
        }
    }
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|language| *language == self)
            .unwrap()
    }
    pub fn t(self, key: &str) -> &str {
        messages()
            .get(key)
            .and_then(|entry| entry.get(self.code()).or_else(|| entry.get("en")))
            .map(String::as_str)
            .unwrap_or(key)
    }
    pub fn format(self, key: &str, values: &[(&str, &str)]) -> String {
        let mut value = self.t(key).to_owned();
        for (name, replacement) in values {
            value = value.replace(&format!("{{{name}}}"), replacement);
        }
        value
    }
    pub fn category(self, value: &str) -> &str {
        match value {
            "Streaming" => self.t("categoryStreaming"),
            "Music" => self.t("categoryMusic"),
            "Gaming" => self.t("categoryGaming"),
            "Utilities" => self.t("categoryUtilities"),
            _ => self.t("categoryUtilities"),
        }
    }
    pub fn clock(self, now: OffsetDateTime, hour24: bool, seconds: bool) -> String {
        let base = self.clock_base(now, hour24);
        if !seconds {
            return base;
        }
        let tail = format!(":{:02}", now.second());
        match base.find(' ') {
            Some(space) => format!("{}{tail}{}", &base[..space], &base[space..]),
            None => format!("{base}{tail}"),
        }
    }
    fn clock_base(self, now: OffsetDateTime, hour24: bool) -> String {
        if hour24 {
            return if self == Self::En {
                format!("{:02}:{:02}", now.hour(), now.minute())
            } else {
                format!("{}:{:02}", now.hour(), now.minute())
            };
        }
        let hour = match now.hour() % 12 {
            0 => 12,
            value => value,
        };
        if matches!(self, Self::Zh | Self::ZhTw) {
            format!(
                "{}{hour}:{:02}",
                if now.hour() >= 12 { "下午" } else { "上午" },
                now.minute()
            )
        } else {
            format!(
                "{hour}:{:02} {}",
                now.minute(),
                if now.hour() >= 12 { "PM" } else { "AM" }
            )
        }
    }
}
pub fn deserialize_language<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Language, D::Error> {
    Ok(
        match serde_json::Value::deserialize(deserializer)?.as_str() {
            Some("de") => Language::De,
            Some("zh") => Language::Zh,
            Some("zh-tw" | "zh-TW" | "zh-Hant") => Language::ZhTw,
            Some("es") => Language::Es,
            Some("pt") => Language::Pt,
            Some("it") => Language::It,
            Some("ru") => Language::Ru,
            Some("pl") => Language::Pl,
            Some("fr") => Language::Fr,
            Some("vi") => Language::Vi,
            Some("nl") => Language::Nl,
            Some("sv") => Language::Sv,
            _ => Language::En,
        },
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn traditional_chinese_follows_simplified_and_preserves_locale_preferences() {
        use super::Language;
        assert_eq!(Language::ALL[Language::Zh.index() + 1], Language::ZhTw);
        assert_eq!(Language::ZhTw.name(), "繁體中文");
        assert_eq!(Language::ZhTw.code(), "zh-tw");
        for code in ["zh-tw", "zh-TW", "zh-Hant"] {
            let preferences: crate::model::Preferences =
                serde_json::from_value(serde_json::json!({"language": code})).unwrap();
            assert_eq!(preferences.language, Language::ZhTw);
            assert_eq!(
                serde_json::to_value(&preferences).unwrap()["language"],
                "zh-tw"
            );
        }
        let now = time::OffsetDateTime::parse(
            "2026-10-21T20:41:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        assert_eq!(Language::ZhTw.clock(now, false, false), "下午8:41");
        assert_eq!(Language::ZhTw.clock(now, false, true), "下午8:41:00");
    }

    #[test]
    fn bundled_cjk_font_covers_every_traditional_chinese_message() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/noto-sans-sc.ttf"
        ))
        .unwrap();
        let font = crate::font::Font::from_bytes(bytes).unwrap();
        let extra = "繁體中文一二三四五六日月星期上午下午";
        for character in super::messages()
            .values()
            .flat_map(|entry| entry["zh-tw"].chars())
            .chain(extra.chars())
        {
            if ('\u{3000}'..='\u{9fff}').contains(&character) {
                assert_ne!(
                    font.lookup_glyph_index(character),
                    0,
                    "Missing Traditional Chinese glyph {character}"
                );
            }
        }
    }

    #[test]
    fn homebrew_description_lists_every_supported_language() {
        let description = include_str!("../assets/homebrew-description.html");
        for language in super::Language::ALL {
            let name = match language {
                super::Language::En => "English",
                super::Language::De => "German",
                super::Language::Zh | super::Language::ZhTw => "Simplified / Traditional Chinese",
                super::Language::Es => "Spanish",
                super::Language::Pt => "Portuguese (Brazil)",
                super::Language::It => "Italian",
                super::Language::Ru => "Russian",
                super::Language::Pl => "Polish",
                super::Language::Fr => "French",
                super::Language::Vi => "Vietnamese",
                super::Language::Nl => "Dutch",
                super::Language::Sv => "Swedish",
            };
            assert!(
                description.contains(name),
                "Homebrew description is missing {name}"
            );
        }
    }
    use super::*;
    #[test]
    fn every_message_has_all_languages_and_no_empty_values() {
        for (key, entry) in messages() {
            for language in Language::ALL {
                assert!(
                    !entry
                        .get(language.code())
                        .unwrap_or_else(|| panic!("Missing {key} / {}", language.code()))
                        .is_empty()
                );
            }
        }
    }
    #[test]
    fn every_translation_preserves_the_english_message_placeholders() {
        fn placeholders(message: &str) -> std::collections::BTreeSet<&str> {
            message
                .split('{')
                .skip(1)
                .map(|part| {
                    part.split_once('}')
                        .expect("Unclosed message placeholder")
                        .0
                })
                .collect()
        }

        for (key, entry) in messages() {
            let expected = placeholders(&entry["en"]);
            for language in Language::ALL {
                assert_eq!(
                    placeholders(&entry[language.code()]),
                    expected,
                    "Mismatched placeholders in {key} / {}",
                    language.code()
                );
            }
        }
    }
    #[test]
    fn localization_keeps_user_provided_names_intact() {
        assert_eq!(
            Language::De.format("favoriteAdded", &[("app", "Netflix")]),
            "Netflix zu Favoriten hinzugefügt."
        );
        assert_eq!(
            Language::Es.format("favoriteAdded", &[("app", "Netflix")]),
            "Netflix añadida a favoritos."
        );
        assert_eq!(
            Language::Pt.format("favoriteAdded", &[("app", "Netflix")]),
            "Netflix adicionado aos favoritos."
        );
        assert_eq!(
            Language::Zh.format("hidden", &[("count", "3")]),
            "已隐藏（3）"
        );
        assert_eq!(
            Language::Ru.format("favoriteAdded", &[("app", "Netflix")]),
            "Netflix: добавлено в избранное."
        );
        assert_eq!(
            Language::Pl.format("favoriteAdded", &[("app", "Netflix")]),
            "Dodano Netflix do ulubionych."
        );
        assert_eq!(
            Language::Fr.format("favoriteAdded", &[("app", "Netflix")]),
            "Netflix fait maintenant partie de vos favoris."
        );
        assert_eq!(
            Language::Vi.format("favoriteAdded", &[("app", "Netflix")]),
            "Đã thêm Netflix vào mục ưa thích."
        );
        assert_eq!(
            Language::Nl.format("favoriteAdded", &[("app", "Netflix")]),
            "Netflix toegevoegd aan favorieten."
        );
        assert_eq!(Language::Nl.category("Gaming"), "Games");
        assert_eq!(
            Language::Sv.format("favoriteAdded", &[("app", "Netflix")]),
            "Netflix har lagts till bland favoriterna."
        );
        assert_eq!(Language::Sv.category("Gaming"), "Spel");
        assert_eq!(Language::De.category("Gaming"), "Spiele");
    }
    #[test]
    fn clock_follows_the_selected_locale() {
        let now = OffsetDateTime::parse(
            "2026-10-21T20:41:00+02:00",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        assert_eq!(Language::Ru.clock(now, true, false), "20:41");
        assert_eq!(Language::Pl.clock(now, true, false), "20:41");
        assert_eq!(Language::Fr.clock(now, true, false), "20:41");
        assert_eq!(Language::Vi.clock(now, true, false), "20:41");
        assert_eq!(Language::Nl.clock(now, true, false), "20:41");
        assert_eq!(Language::Sv.clock(now, true, false), "20:41");
        assert_eq!(Language::Zh.clock(now, false, false), "下午8:41");
        assert_eq!(Language::Zh.clock(now, false, true), "下午8:41:00");
        assert_eq!(Language::En.clock(now, true, false), "20:41");
        assert_eq!(Language::En.clock(now, true, true), "20:41:00");
        assert_eq!(Language::En.clock(now, false, true), "8:41:00 PM");
    }
}
