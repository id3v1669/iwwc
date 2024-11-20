pub fn check_image() {
    let config = crate::data::shared_data::CONFIG.lock().unwrap();

    const DEFAULT_ICON: &[u8] = include_bytes!("../../assets/testing/default.svg");

    let path = &config.default_icon_dir;
    if !std::path::Path::new(&path).exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            log::error!("Failed to create a default icon directory: {}", e);
            std::process::exit(1);
        }
    }
    if !std::path::Path::new(&path.join("default.svg")).exists() {
        if let Err(e) = std::fs::write(&path.join("default.svg"), DEFAULT_ICON) {
                log::error!("Failed to create a default icon: {}", e);
                std::process::exit(1);
        }
    }
    else {
        log::info!("Default icon already exists");
    }
}