use crate::i18n::filter::TString;
use crate::widget::Filter;
use crate::widget::filter::{BookHighlights, BooksHighlights, Highlights, Info};
use anyhow::{Context, Result, anyhow};

impl Filter {
    // to update the list of filtered books
    // and create highlights
    pub(super) fn update(&self) -> Result<()> {
        if self.input.lock().unwrap().is_empty() {
            *self.filtered_uuids.lock().unwrap() = self.books_info.keys().cloned().collect();
            *self.books_highlights.lock().unwrap() = BooksHighlights::new();
            if !self.filtered_uuids.lock().unwrap().is_empty() {
                self.table_state.lock().unwrap().select(Some(0));
            } else {
                self.table_state.lock().unwrap().select(None);
            }
            return Ok(());
        }

        let input_lower_case = self.input.lock().unwrap().to_lowercase().replace(" ", "");
        let mut errors: Vec<anyhow::Error> = Vec::new();

        // get iterator of result
        let results: Vec<(String, BookHighlights)> = self
            .books_info
            .iter()
            .filter_map(|(uuid, info)| {
                match self.for_book_find_matches_and_create_highlights(info, &input_lower_case) {
                    Ok((found_match, book_highlights)) => {
                        if found_match {
                            Some(Ok((uuid.clone(), book_highlights)))
                        } else {
                            None // no match
                        }
                    }
                    Err(e) => {
                        // collect error
                        errors.push(e.context(format!("fail to process book uuid: {}", uuid)));
                        Some(Err(())) // return a mark (will be filterd later)
                    }
                }
            })
            .filter_map(Result::ok) // only take ok result
            .collect();

        // return err
        if !errors.is_empty() {
            let mut error_messages = String::from("errors occurred during book filtering:\n");
            for e in errors {
                error_messages.push_str(&format!("- {:?}\n", e));
            }
            return Err(anyhow::anyhow!(error_messages));
        }

        *self.filtered_uuids.lock().unwrap() =
            results.iter().map(|(uuid, _)| uuid.clone()).collect();
        *self.books_highlights.lock().unwrap() = results.into_iter().collect();

        if !self.filtered_uuids.lock().unwrap().is_empty() {
            self.table_state.lock().unwrap().select(Some(0));
        } else {
            self.table_state.lock().unwrap().select(None);
        }

        Ok(())
    }

    // for a book
    fn for_book_find_matches_and_create_highlights(
        &self,
        info: &Info,
        input: &str,
    ) -> Result<(bool, BookHighlights)> {
        for (name, version) in info {
            let input_i18n = if name == "default" {
                Ok(input.to_string())
            } else {
                self.i18n_handler
                    .translators
                    .get(name)
                    .ok_or_else(|| anyhow!("can't find translator named: {:?}", name))
                    .and_then(|translator| {
                        translator.trans_input(&input).context(format!(
                            "failed to translate input for translator '{}'",
                            name
                        ))
                    })
            }
            .with_context(|| format!("failed to translate input for book version '{}'", name))?;

            let book_highlights = {
                let inputs = vec![input_i18n];
                let (title_highlights, series_highlights, tags_highlights, authors_highlights) = (
                    self.for_tstring_find_matches_and_create_highlights(&version.title, &inputs)
                        .context(format!(
                            "failed to create highlights for title of book version '{:?}'",
                            version.title
                        ))?,
                    self.for_tstring_find_matches_and_create_highlights(&version.series, &inputs)
                        .context(format!(
                            "failed to create highlights for series of book version '{:?}'",
                            version.series
                        ))?,
                    self.for_tstring_find_matches_and_create_highlights(&version.tags, &inputs)
                        .context(format!(
                            "failed to create highlights for tags of book version '{:?}'",
                            version.tags
                        ))?,
                    self.for_tstring_find_matches_and_create_highlights(&version.authors, &inputs)
                        .context(format!(
                            "Failed to create highlights for authors of book version '{:?}'",
                            version.authors
                        ))?,
                );

                BookHighlights {
                    title: title_highlights,
                    series: series_highlights,
                    tags: tags_highlights,
                    authors: authors_highlights,
                }
            };
            let found_match = book_highlights.title.get(0).map_or(false, |&(b, _, _)| b)
                || book_highlights.series.get(0).map_or(false, |&(b, _, _)| b)
                || book_highlights.tags.get(0).map_or(false, |&(b, _, _)| b)
                || book_highlights.authors.get(0).map_or(false, |&(b, _, _)| b);
            if found_match {
                return Ok((true, book_highlights));
            }
        }
        Ok((
            false,
            BookHighlights {
                title: vec![(false, 0, 0)],
                tags: vec![(false, 0, 0)],
                series: vec![(false, 0, 0)],
                authors: vec![(false, 0, 0)],
            },
        ))
    }

    /// find matches and create highlights for a TString
    /// why set query as vec of string?
    /// leaving space for future extension
    fn for_tstring_find_matches_and_create_highlights(
        &self,
        translation: &TString,
        query: &Vec<String>,
    ) -> Result<Highlights> {
        let (full_str, str_index) = translation;
        let char_to_token_map: Vec<usize> = str_index
            .windows(2)
            .enumerate()
            .map(|(index, indices)| vec![index; indices[1] - indices[0]])
            .flatten()
            .collect();

        // make sure create a correct map (at least have the same length, right?)
        if full_str.chars().count() != char_to_token_map.len() {
            return Err(anyhow!(
                "the length of the full_str ({}) and the char_to_token_map ({}) must be equal. this indicates an internal data inconsistency.",
                full_str.chars().count(),
                char_to_token_map.len()
            ));
        }

        // result of this match
        let mut match_results = Vec::new();

        for needle in query {
            if let Some((match_start_byte_idx, matched_str)) = full_str.match_indices(needle).next()
            {
                // get char index not byte index
                let start_char_idx = full_str[..match_start_byte_idx].chars().count();
                let end_char_idx = start_char_idx + matched_str.chars().count();

                if end_char_idx > 0 && end_char_idx <= char_to_token_map.len() {
                    let start_token_idx = char_to_token_map[start_char_idx];
                    let end_token_idx = char_to_token_map[end_char_idx - 1];

                    // end index not inculded so +1
                    match_results.push((true, start_token_idx, end_token_idx + 1));
                } else {
                    match_results.push((false, 0, 0));
                }
            } else {
                match_results.push((false, 0, 0));
            }
        }

        Ok(match_results)
    }
}
