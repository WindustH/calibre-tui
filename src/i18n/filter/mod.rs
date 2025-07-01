use std::collections::HashMap;
// use crate::utils::book::Metadata;
use anyhow::Result;

mod pinyin;

// translated string with token indices
pub type TString=(String, Vec<usize>);
// inermediate layer between two languages
pub trait IR: Send + Sync + 'static {
    // translate book's info
    fn trans_book_info(&self, s: &str) -> Result<TString>;

    // translate input string
    fn trans_input(&self, s:&str)->Result<String>;
    // is this translator enabled?
    fn is_enabled(&self) -> bool;
}


// i18n handler
pub struct Handler {
    pub translators: HashMap<String, Box<dyn IR>>,
}

impl Handler {
    pub fn new(config: &crate::config::i18n::Filter) -> Result<Self> {
        // modify here to add more translators

        // match (pinyin::Pinyin::new(&config.pinyin),translator::Translator::new(&config.translator)) {
        //     (Ok(pinyin_handler),Ok(new_handler)) => {
        //         let mut translators = HashMap::<String, Box<dyn IR>>::new();

        //         translators.insert("pinyin".to_string(), Box::new(pinyin_handler));
        //         translators.insert("translator".to_string(), Box::new(translator_handler));

        //         Ok(Self { translators })
        //     }
        //     ...
        // }

        match pinyin::Pinyin::new(&config.pinyin) {
            Ok(pinyin_handler) => {
                let mut translators = HashMap::<String, Box<dyn IR>>::new();

                translators.insert("pinyin".to_string(), Box::new(pinyin_handler));

                Ok(Self { translators })
            }
            Err(e) => Err(e),
        }
    }
}