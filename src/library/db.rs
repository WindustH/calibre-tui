//! Read-only loading of Calibre's `metadata.db`.

use super::Book;
use anyhow::{Context, Result};
use rusqlite::{Connection, ErrorCode, OpenFlags, Row};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Separator for aggregated list columns (ASCII unit separator, `char(31)`
/// in SQL), which does not occur in Calibre names.
const LIST_SEPARATOR: char = '\u{1f}';

/// One row per book. The opened file is the most recently added format,
/// stored by Calibre at `<library>/<books.path>/<data.name>.<format>`.
const BOOKS_QUERY: &str = "
  SELECT
    b.title AS title,
    b.path AS dir,
    d.name AS file_name,
    d.format AS file_format,
    (SELECT group_concat(name, char(31)) FROM (
      SELECT a.name FROM books_authors_link bal JOIN authors a ON a.id = bal.author
      WHERE bal.book = b.id ORDER BY bal.id
    )) AS authors,
    s.name AS series,
    (SELECT group_concat(format, char(31)) FROM data WHERE book = b.id) AS formats,
    (SELECT group_concat(t.name, char(31)) FROM tags t JOIN books_tags_link btl ON t.id = btl.tag
      WHERE btl.book = b.id) AS tags
  FROM books b
  LEFT JOIN data d ON d.id = (SELECT id FROM data WHERE book = b.id ORDER BY id DESC LIMIT 1)
  LEFT JOIN books_series_link bsl ON bsl.book = b.id
  LEFT JOIN series s ON s.id = bsl.series
  ORDER BY b.sort
";

/// Load all books without ever writing to the database. Calibre may hold a
/// write lock while it is running; if the lock does not clear quickly the
/// file is read as immutable (no locking) instead of failing.
pub fn load_books(library: &Path) -> Result<Vec<Book>> {
  let db_path = library.join("metadata.db");
  let result = match read_books(open_read_only(&db_path), library) {
    Err(error) if is_lock_error(&error) => read_books(open_immutable(&db_path), library),
    result => result,
  };
  result.with_context(|| format!("failed to read Calibre database '{}'", db_path.display()))
}

fn open_read_only(db_path: &Path) -> rusqlite::Result<Connection> {
  let connection = Connection::open_with_flags(
    db_path,
    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
  )?;
  connection.busy_timeout(Duration::from_secs(1))?;
  Ok(connection)
}

fn open_immutable(db_path: &Path) -> rusqlite::Result<Connection> {
  Connection::open_with_flags(
    immutable_uri(db_path),
    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX,
  )
}

fn is_lock_error(error: &rusqlite::Error) -> bool {
  matches!(
    error.sqlite_error_code(),
    Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
  )
}

/// `file:` URI with `immutable=1`; every byte outside the unreserved set is
/// percent-encoded so spaces, `?`, `#`, `%` and non-UTF-8 bytes survive.
fn immutable_uri(db_path: &Path) -> String {
  let path = std::path::absolute(db_path).unwrap_or_else(|_| db_path.to_path_buf());
  #[cfg(windows)]
  let bytes = path.to_string_lossy().replace('\\', "/").into_bytes();
  #[cfg(not(windows))]
  let bytes = path.into_os_string().into_encoded_bytes();

  // An empty authority keeps `//server/share` and `C:/...` inside the path.
  let mut uri = String::from("file://");
  if !bytes.starts_with(b"/") {
    uri.push('/');
  }
  for byte in bytes {
    if byte.is_ascii_alphanumeric() || b"-._~/:".contains(&byte) {
      uri.push(char::from(byte));
    } else {
      let _ = write!(uri, "%{byte:02X}");
    }
  }
  uri.push_str("?mode=ro&immutable=1");
  uri
}

fn read_books(
  connection: rusqlite::Result<Connection>,
  library: &Path,
) -> rusqlite::Result<Vec<Book>> {
  let connection = connection?;
  let mut statement = connection.prepare(BOOKS_QUERY)?;
  statement
    .query_map([], |row| book_from_row(row, library))?
    .collect()
}

fn book_from_row(row: &Row<'_>, library: &Path) -> rusqlite::Result<Book> {
  let text = |column: &str| -> rusqlite::Result<String> {
    Ok(row.get::<_, Option<String>>(column)?.unwrap_or_default())
  };

  let dir = text("dir")?;
  let file_name = row.get::<_, Option<String>>("file_name")?;
  let file_format = row.get::<_, Option<String>>("file_format")?;
  let path = file_name
    .zip(file_format)
    .map(|(name, format)| book_file_path(library, &dir, &name, &format));

  // Calibre stores commas inside author names as `|`.
  let mut authors = split_list(&text("authors")?.replace('|', ","));
  if authors.is_empty() {
    authors.push("Unknown Author".to_string());
  }

  Ok(Book {
    title: text("title")?,
    authors,
    series: text("series")?,
    formats: split_list(&text("formats")?),
    tags: split_list(&text("tags")?),
    path,
  })
}

/// Build the file path with native separators (Calibre stores `/`).
fn book_file_path(library: &Path, dir: &str, name: &str, format: &str) -> PathBuf {
  let mut path = library.to_path_buf();
  path.extend(dir.split('/').filter(|part| !part.is_empty()));
  path.push(format!("{name}.{}", format.to_lowercase()));
  path
}

fn split_list(value: &str) -> Vec<String> {
  value
    .split(LIST_SEPARATOR)
    .map(str::trim)
    .filter(|item| !item.is_empty())
    .map(str::to_string)
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  const SCHEMA: &str = "
    CREATE TABLE books (id INTEGER PRIMARY KEY, title TEXT, sort TEXT, path TEXT NOT NULL DEFAULT '');
    CREATE TABLE authors (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
    CREATE TABLE books_authors_link (id INTEGER PRIMARY KEY, book INTEGER, author INTEGER);
    CREATE TABLE series (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
    CREATE TABLE books_series_link (id INTEGER PRIMARY KEY, book INTEGER, series INTEGER);
    CREATE TABLE tags (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
    CREATE TABLE books_tags_link (id INTEGER PRIMARY KEY, book INTEGER, tag INTEGER);
    CREATE TABLE data (id INTEGER PRIMARY KEY, book INTEGER, format TEXT, name TEXT);
    INSERT INTO books VALUES (1, 'Dune', 'Dune', 'Frank Herbert/Dune (1)');
    INSERT INTO books VALUES (2, 'No Files', 'No Files', 'X/No Files (2)');
    INSERT INTO authors VALUES (1, 'Herbert| Frank'), (2, 'Anderson');
    INSERT INTO books_authors_link VALUES (1, 1, 2), (2, 1, 1);
    INSERT INTO series VALUES (1, 'Dune Chronicles');
    INSERT INTO books_series_link VALUES (1, 1, 1);
    INSERT INTO tags VALUES (1, 'Sci-Fi, Classic');
    INSERT INTO books_tags_link VALUES (1, 1, 1);
    INSERT INTO data VALUES (1, 1, 'PDF', 'Dune - Frank Herbert'), (2, 1, 'EPUB', 'Dune - Frank Herbert');
  ";

  fn fixture_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("calibre-tui-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Connection::open(dir.join("metadata.db"))
      .unwrap()
      .execute_batch(SCHEMA)
      .unwrap();
    dir
  }

  #[test]
  fn loads_metadata_and_latest_format() {
    let library = fixture_dir("load");
    let books = load_books(&library).unwrap();
    std::fs::remove_dir_all(&library).unwrap();

    assert_eq!(books.len(), 2);
    let dune = &books[0];
    assert_eq!(dune.authors, ["Anderson", "Herbert, Frank"]);
    assert_eq!(dune.series, "Dune Chronicles");
    assert_eq!(dune.formats, ["PDF", "EPUB"]);
    assert_eq!(dune.tags, ["Sci-Fi, Classic"]);
    let expected = library
      .join("Frank Herbert")
      .join("Dune (1)")
      .join("Dune - Frank Herbert.epub");
    assert_eq!(dune.path, Some(expected));

    let no_files = &books[1];
    assert_eq!(no_files.path, None);
    assert_eq!(no_files.authors, ["Unknown Author"]);
  }

  #[test]
  fn reads_while_another_process_holds_an_exclusive_lock() {
    let library = fixture_dir("locked dir #%");
    let writer = Connection::open(library.join("metadata.db")).unwrap();
    writer.execute_batch("BEGIN EXCLUSIVE").unwrap();

    let books = load_books(&library);
    writer.execute_batch("ROLLBACK").unwrap();
    drop(writer);
    std::fs::remove_dir_all(&library).unwrap();

    assert_eq!(books.unwrap().len(), 2);
  }

  #[test]
  fn never_creates_a_missing_database() {
    let library = std::env::temp_dir().join(format!("calibre-tui-missing-{}", std::process::id()));
    std::fs::create_dir_all(&library).unwrap();
    assert!(load_books(&library).is_err());
    assert!(!library.join("metadata.db").exists());
    std::fs::remove_dir_all(&library).unwrap();
  }
}
