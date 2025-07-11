use crate::utils::book::{Book, Books};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;

/// load book list from calibre metadata.db
pub fn load_books_from_db(library_path: &PathBuf) -> Result<Books> {
    let db_path = library_path.join("metadata.db");
    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open Calibre database: {:?}", db_path))?;

    let mut stmt = conn.prepare(
        "
        SELECT
            b.uuid AS uuid,
            b.title AS title,
            b.timestamp AS timestamp,
            b.pubdate AS pubdate,
            b.last_modified AS last_modified,
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
        let uuid: String = row.get("uuid")?;

        // convert string to time struct
        let timestamp_str: String = row.get("timestamp")?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        let last_modified_str: String = row.get("last_modified")?;
        let last_modified = DateTime::parse_from_rfc3339(&last_modified_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        let pubdate: Option<DateTime<Utc>> = row
            .get::<_, Option<String>>("pubdate")?
            .map(|s| {
                DateTime::parse_from_rfc3339(&s).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            })
            .transpose()?
            .map(|dt| dt.with_timezone(&Utc));
        // get title with fallback ""
        let title: String = row
            .get::<&str, Option<String>>("title")?
            .unwrap_or_default();
        // get relative_path with fallback ""
        let relative_path: String = row
            .get::<&str, Option<String>>("relative_path")?
            .unwrap_or_default();
        // get full path
        let full_path = if relative_path.is_empty() {
            PathBuf::from("")
        } else {
            library_path.join(&relative_path)
        };
        // get authors with fallback "Unknown Author"
        // let authors: String = row
        //     .get::<&str, String>("authors")
        //     .unwrap_or_else(|_| "Unknown Author".to_string());
        // get series with fallback ""
        let series: String = row
            .get::<&str, Option<String>>("series")?
            .unwrap_or_default();

        // get tags (no more connected into a string) with fallback ""
        let tags: Vec<String> = row
            .get::<&str, Option<String>>("tags")?
            .unwrap_or_default()
            .split(',') // split by ','
            .map(|s| s.trim().to_string()) // remove space and turn into string
            .filter(|s| !s.is_empty()) // filter empty string
            .collect();

        // get tags (connected into a string) with fallback ""
        let authors: Vec<String> = row
            .get::<&str, String>("authors")
            .unwrap_or_else(|_| "Unknown Author".to_string())
            .split('&') // split by '&'
            .map(|s| s.trim().to_string()) // remove space and turn into string
            .filter(|s| !s.is_empty()) // filter empty string
            .collect();

        let book = Book {
            path: full_path,
            timestamp,
            pubdate,
            last_modified,
            title,
            authors,
            series,
            tags,
        };

        Ok((uuid, book))
    })?;

    // collect as hashmap
    let book_map: HashMap<String, Book> = book_iter.collect::<Result<_, _>>()?;

    Ok(book_map)
}

/// get book's data from metadata.db by uuid
pub fn get_book_by_uuid_from_db(library_path: &PathBuf, uuid: &str) -> Result<Option<Book>> {
    let db_path = library_path.join("metadata.db");
    let conn = Connection::open(&db_path)
        .with_context(|| format!("failed to open calibre database: {:?}", db_path))?;
    let mut stmt = conn.prepare(
        "
        SELECT
            b.uuid AS uuid,
            b.title AS title,
            b.timestamp AS timestamp,
            b.pubdate AS pubdate,
            b.last_modified AS last_modified,
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
        WHERE
            b.uuid = ?
        ",
    )?;
    let mut rows = stmt.query([uuid])?;
    if let Some(row) = rows.next()? {
        let _ = row.get::<_, String>("uuid")?;
        let timestamp_str: String = row.get("timestamp")?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        let last_modified_str: String = row.get("last_modified")?;
        let last_modified = DateTime::parse_from_rfc3339(&last_modified_str)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc);

        let pubdate: Option<DateTime<Utc>> = row
            .get::<_, Option<String>>("pubdate")?
            .map(|s| {
                DateTime::parse_from_rfc3339(&s).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            })
            .transpose()?
            .map(|dt| dt.with_timezone(&Utc));

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
            .get::<&str, String>("authors")
            .unwrap_or_else(|_| "Unknown Author".to_string())
            .split('&')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let book = Book {
            title,
            path: full_path,
            timestamp,
            pubdate,
            last_modified,
            authors,
            series,
            tags,
        };
        Ok(Some(book))
    } else {
        Ok(None)
    }
}
