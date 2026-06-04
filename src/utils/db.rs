use crate::utils::book::Book;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// load book list from calibre metadata.db
pub fn load_books_from_db(library_path: &Path) -> Result<Vec<Book>> {
  let db_path = library_path.join("metadata.db");
  let conn = Connection::open(&db_path)
    .with_context(|| format!("Failed to open Calibre database: {:?}", db_path))?;

  let mut stmt = conn.prepare(
        "
        SELECT
            b.title AS title,
            b.path || '/' || (SELECT name FROM data WHERE book = b.id ORDER BY id DESC LIMIT 1) || '.' || lower((SELECT format FROM data WHERE book = b.id ORDER BY id DESC LIMIT 1)) AS relative_path,
            (SELECT GROUP_CONCAT(a.name, '&') FROM authors a JOIN books_authors_link bal ON a.id = bal.author WHERE bal.book = b.id) AS authors,
            s.name AS series,
            (SELECT GROUP_CONCAT(t.name, ',') FROM tags t JOIN books_tags_link btl ON t.id = btl.tag WHERE btl.book = b.id) AS tags
        FROM
            books b
        LEFT JOIN
            books_series_link bsl ON b.id = bsl.book
        LEFT JOIN
            series s ON bsl.series = s.id
        ORDER BY
            b.sort;
        ",
    )?;

  let book_iter = stmt.query_map([], |row| {
    let title: String = row
      .get::<&str, Option<String>>("title")?
      .unwrap_or_default();
    let relative_path: String = row
      .get::<&str, Option<String>>("relative_path")?
      .unwrap_or_default();
    let full_path = if relative_path.is_empty() {
      PathBuf::from("")
    } else {
      library_path.join(&relative_path)
    };
    let series: String = row
      .get::<&str, Option<String>>("series")?
      .unwrap_or_default();
    let tags: Vec<String> = row
      .get::<&str, Option<String>>("tags")?
      .unwrap_or_default()
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    let authors: Vec<String> = row
      .get::<&str, Option<String>>("authors")?
      .unwrap_or_else(|| "Unknown Author".to_string())
      .split('&')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    let book = Book {
      path: full_path,
      title,
      authors,
      series,
      tags,
    };

    Ok(book)
  })?;

  Ok(book_iter.collect::<Result<Vec<_>, _>>()?)
}
