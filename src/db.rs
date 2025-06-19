use crate::app::Book;
use crate::pinyin::get_simple_pinyin;
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

/// 从 Calibre 的 metadata.db 文件中加载书籍列表
// The function now accepts a flag to conditionally generate pinyin.
pub fn load_books_from_db(library_path: &Path, pinyin_enabled: bool) -> Result<Vec<Book>> {
    let db_path = library_path.join("metadata.db");
    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open Calibre database: {:?}", db_path))?;

    let mut stmt = conn.prepare(
        "
        SELECT
            b.title,
            (SELECT GROUP_CONCAT(a.name, ' & ') FROM authors a JOIN books_authors_link bal ON a.id = bal.author WHERE bal.book = b.id) as author,
            s.name as series,
            (SELECT GROUP_CONCAT(t.name, ', ') FROM tags t JOIN books_tags_link btl ON t.id = btl.tag WHERE btl.book = b.id) as tags,
            b.path || '/' || (SELECT name FROM data WHERE book = b.id LIMIT 1) || '.' || lower((SELECT format FROM data WHERE book = b.id LIMIT 1))
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
        let title: String = row.get(0)?;
        let author: String = row.get(1).unwrap_or_else(|_| "Unknown Author".to_string());
        let series: String = row.get(2).unwrap_or_else(|_| "".to_string());
        let tags: String = row.get(3).unwrap_or_else(|_| "".to_string());

        // Conditionally generate pinyin to save performance if the feature is disabled.
        let (title_pinyin, author_pinyin, series_pinyin, tags_pinyin) = if pinyin_enabled {
            (
                get_simple_pinyin(&title),
                get_simple_pinyin(&author),
                get_simple_pinyin(&series),
                get_simple_pinyin(&tags),
            )
        } else {
            // If disabled, store empty strings to avoid unnecessary computation.
            (String::new(), String::new(), String::new(), String::new())
        };

        Ok(Book {
            title,
            author,
            series,
            tags,
            path: row.get(4)?,
            title_pinyin,
            author_pinyin,
            series_pinyin,
            tags_pinyin,
        })
    })?;

    let books: Result<Vec<Book>, _> = book_iter.collect();
    Ok(books?)
}
