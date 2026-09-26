//! Command prompt commands (`sort`, `help`) and their completion.

use crate::library::BookField;
use crate::sort::{SortDirection, SortSpec};
use framework_tui::{CommandCompletion, current_word_start, filter_completion_candidates};

const COMMAND_NAMES: &[&str] = &["help", "sort"];
const SORT_FIELDS: &[&str] = &["title", "authors", "series", "formats", "tags"];
const SORT_DIRECTIONS: &[&str] = &["asc", "desc"];

#[derive(Debug, PartialEq)]
pub enum Command {
  Sort(SortSpec),
  Help,
}

/// Parse a prompt command or keymap action; the leading `:` is optional.
/// Errors are user-facing messages.
pub fn parse(input: &str) -> Result<Command, String> {
  let mut parts = input.trim().trim_start_matches(':').split_whitespace();
  match parts.next() {
    Some("sort") => SortSpec::parse(&parts.collect::<Vec<_>>())
      .map(Command::Sort)
      .map_err(|error| error.to_string()),
    Some("help") if parts.next().is_none() => Ok(Command::Help),
    Some(other) => Err(format!("unknown command: {other}")),
    None => Err("empty command".to_string()),
  }
}

/// Completion for the word before `cursor` (a byte offset into `input`).
pub fn completion(input: &str, cursor: usize) -> Option<CommandCompletion> {
  let cursor = cursor.min(input.len());
  let before_cursor = input.get(..cursor)?;
  let normalized = before_cursor.trim_start_matches(':');
  let tokens = normalized.split_whitespace().collect::<Vec<_>>();
  let ends_with_space = normalized.chars().last().is_some_and(char::is_whitespace);
  let word_start = current_word_start(input, cursor);
  let word = if ends_with_space {
    ""
  } else {
    input.get(word_start..cursor).unwrap_or_default()
  };

  if tokens.is_empty() || (tokens.len() == 1 && !ends_with_space) {
    // Keep an optional leading `:` out of the replaced range so the inline
    // suggestion lines up with what completion inserts.
    let prefix = word.trim_start_matches(':');
    return completion_from_candidates(
      word_start + (word.len() - prefix.len()),
      cursor,
      prefix,
      filter_completion_candidates(COMMAND_NAMES.iter().copied(), prefix),
      ends_with_space,
    );
  }

  match tokens[0] {
    "sort" => sort_completion(&tokens[1..], ends_with_space, word_start, cursor, word),
    _ => None,
  }
}

fn sort_completion(
  args: &[&str],
  ends_with_space: bool,
  word_start: usize,
  cursor: usize,
  word: &str,
) -> Option<CommandCompletion> {
  let completed_args = if ends_with_space {
    args
  } else {
    args.get(..args.len().saturating_sub(1))?
  };
  let candidates = sort_candidates(completed_args)?;
  let replace_start = if ends_with_space { cursor } else { word_start };

  completion_from_candidates(
    replace_start,
    cursor,
    word,
    filter_completion_candidates(candidates, word),
    ends_with_space,
  )
}

fn completion_from_candidates(
  replace_start: usize,
  replace_end: usize,
  prefix: &str,
  candidates: Vec<String>,
  ends_with_space: bool,
) -> Option<CommandCompletion> {
  // A word that is already the only candidate needs no completion.
  if !ends_with_space && candidates.len() == 1 && candidates[0].eq_ignore_ascii_case(prefix) {
    return None;
  }

  Some(CommandCompletion::new(
    replace_start,
    replace_end,
    prefix,
    candidates,
    true,
    0,
  ))
}

/// Candidates for the next `sort` argument, or `None` if the arguments so far
/// are invalid.
fn sort_candidates(completed_args: &[&str]) -> Option<Vec<&'static str>> {
  let mut expecting_field = true;
  for arg in completed_args {
    if expecting_field {
      BookField::parse(arg)?;
      expecting_field = false;
    } else if SortDirection::parse(arg).is_some() {
      expecting_field = true;
    } else {
      BookField::parse(arg)?;
    }
  }

  if expecting_field {
    Some(SORT_FIELDS.to_vec())
  } else {
    Some([SORT_DIRECTIONS, SORT_FIELDS].concat())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn candidates(input: &str) -> Option<Vec<String>> {
    completion(input, input.len()).map(|completion| completion.candidates)
  }

  #[test]
  fn parses_commands_with_optional_colon() {
    assert_eq!(parse(":help"), Ok(Command::Help));
    assert_eq!(
      parse(" sort authors desc "),
      Ok(Command::Sort(
        SortSpec::parse(&["authors", "desc"]).unwrap()
      ))
    );
    assert_eq!(parse(""), Err("empty command".to_string()));
    assert_eq!(parse("open"), Err("unknown command: open".to_string()));
    assert!(parse("sort").is_err());
  }

  #[test]
  fn completes_command_names_and_sort_arguments() {
    assert_eq!(candidates("so"), Some(vec!["sort".to_string()]));
    assert_eq!(candidates("sort"), None);
    assert_eq!(candidates("sort ").unwrap().len(), SORT_FIELDS.len());
    let after_field = candidates("sort title ").unwrap();
    assert!(after_field.contains(&"asc".to_string()) && after_field.contains(&"desc".to_string()));
    assert_eq!(candidates("sort title d"), Some(vec!["desc".to_string()]));
    assert_eq!(candidates("sort rating "), None);
    assert_eq!(candidates("help "), None);
  }

  #[test]
  fn leading_colon_stays_outside_the_completed_word() {
    let completion = completion(":so", 3).unwrap();
    assert_eq!(completion.suggestion_suffix(), "rt");
    let mut buffer = framework_tui::PromptBuffer::new(":so");
    assert!(completion.apply_to(&mut buffer));
    assert_eq!(buffer.input, ":sort ");
  }
}
