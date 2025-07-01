use crate::config;

impl Default for config::I18n {
    fn default() -> Self {
        Self {
            filter: config::i18n::Filter {
                pinyin: config::i18n::filter::Pinyin {
                    enabled: true,
                    fuzzy_enabled: true,
                    fuzzy_groups: vec![
                        vec!["on".to_string(), "ong".to_string()],
                        vec!["an".to_string(), "ang".to_string()],
                        vec!["en".to_string(), "eng".to_string()],
                        vec!["in".to_string(), "ing".to_string()],
                    ],
                },
            },
        }
    }
}
