use core::panic;

use crate::config::validate::app::{is_valid_library_path, possible_library_paths};

// default app config
impl Default for crate::config::App {
    fn default() -> Self {
        let library_path = (|| {
            // check possible library paths
            for path in possible_library_paths() {
                if is_valid_library_path(&path) {
                    return path;
                }
            }
            panic!("no valid library path found");
        })();
        Self {
            library_path,
            default_instance: "filter-and-open".to_string(),
        }
    }
}
