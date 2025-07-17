pub mod filter {
    pub mod table {
        use std::collections::HashSet;

        use crate::config::ui::filter::table::{Column, Columns};
        use serde::Deserialize;
        // raw column to prevent columns with duplicate name
        #[derive(Debug, Deserialize)]
        pub struct RawColumns(Vec<Column>);

        impl<'a> IntoIterator for &'a RawColumns {
            type Item = &'a Column;
            type IntoIter = std::slice::Iter<'a, Column>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        impl TryFrom<RawColumns> for Columns {
            type Error = String;

            fn try_from(raw: RawColumns) -> Result<Self, Self::Error> {
                let mut seen_names = HashSet::new();
                // try to insert label into a hashset
                for column in &raw {
                    if !seen_names.insert(column.label.clone()) {
                        return Err(format!("duplicate column name found: '{}'", column.label));
                    }
                }
                Ok(Columns { 0: raw.0 })
            }
        }
    }
}
