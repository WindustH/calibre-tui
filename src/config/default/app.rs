// default app config
impl Default for crate::config::App {
    fn default() -> Self {
        let library_path =
            match crate::config::validate::app::validate_library_path(&"".to_string()) {
                Ok(path) => path,
                Err(e) => panic!("can't find a default library path {:?}", e),
            };
        Self {
            library_path,
            default_instance: "filter-and-open".to_string(),
        }
    }
}
