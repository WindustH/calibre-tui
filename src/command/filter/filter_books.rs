use crate::command::filter::Info;
use crate::utils::book::Uuids;
use anyhow::Result;

impl super::super::Filter {
    // to update the list of filtered books
    // and create highlights
    fn update_filtered_books(&mut self) {
        if self.input.is_empty() {
            self.filtered_uuids = self.books_info.keys().cloned().collect();
            if !self.filtered_uuids.is_empty() {
                self.table_state.select(Some(0));
            } else {
                self.table_state.select(None);
            }
            return;
        }

        let input_lower_case = self.input.to_lowercase();

        // get the uuids of filtered books
        self.filtered_uuids = self
            .books_info
            .iter()
            .filter_map(
                |(uuid, book)| match self.is_book_match(book, &input_lower_case) {
                    Ok(true) => Some(uuid.clone()),
                    Ok(false) => None,
                    Err(e) => {
                        eprintln!("Error during book match: {:?}", e);
                        None
                    }
                },
            )
            .collect();
        if !self.filtered_uuids.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    // to judge if a book can pass through the filter
    fn is_book_match(&self, info: &Info, input: &str) -> Result<bool> {
        for (name, version) in info {
            let match_translated = if let Some(translator) = self.i18n_handler.translators.get(name)
            {
                let input_i18n = if name == "default" {
                    input.to_string()
                } else {
                    translator.trans_input(&input)?
                };
                version.title.0.contains(&input_i18n)
                    || version.authors.0.contains(&input_i18n)
                    || version.series.0.contains(&input_i18n)
                    || version.tags.0.contains(&input_i18n)
            } else {
                return Err(super::FilterError::MetadataError.into());
            };
            if match_translated {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
