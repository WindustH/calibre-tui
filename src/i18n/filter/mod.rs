use std::collections::HashMap;
// use crate::utils::book::Metadata;
use anyhow::Result;

mod pinyin;

pub type Translation=(String, Vec<usize>);
// inermediate layer between two languages
pub trait IR: Send + Sync + 'static {
    // fn translate_metadata(&self, metadata: &Metadata) -> Result<Metadata>;
    fn trans_book_info(&self, s: &str) -> Result<Translation>;
    // no need for clone anymore

    // translate input string
    fn trans_input(&self, s:&str)->Result<String>;
    // fn box_clone(&self) -> Box<dyn IR>;
    fn is_enabled(&self) -> bool;
}



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

// no need for clone anymore

// impl Clone for I18nHandler {
//     fn clone(&self) -> Self {
//         let mut cloned_translators = HashMap::new();
//         for (key, translator_box) in self.translators.iter() {
//             let cloned_key = key.clone();
//             let cloned_box = translator_box.box_clone();
//             cloned_translators.insert(cloned_key, cloned_box);
//         }
//         I18nHandler {
//             translators: cloned_translators,
//         }
//     }
// }
