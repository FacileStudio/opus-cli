#[cfg(unix)]
#[test]
fn a_snapshot_round_trips_through_the_xdg_cache() {
    use opus_cli::cache;
    use opus_cli::config::OpusConfig;
    use opus_cli::tui::app::state::App;
    use std::fs;

    let root = std::env::temp_dir().join(format!("opus-cli-cache-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    std::env::set_var("XDG_CACHE_HOME", &root);

    let mut app = App::new_with_config(OpusConfig::default(), "Inbox".to_string());
    app.project_map.insert("1".to_string(), "Inbox".to_string());
    app.label_map.insert("2".to_string(), "bug".to_string());

    let snapshot = cache::Snapshot::from_app(&app);
    cache::save("http://localhost:1337", "42", &snapshot).unwrap();

    let path = root.join("opus/http-localhost-1337-42.json");
    assert!(path.is_file(), "cache file missing at {}", path.display());

    let loaded = cache::load("http://localhost:1337", "42").expect("cache should load");
    assert_eq!(
        loaded.project_map.get("1").map(String::as_str),
        Some("Inbox")
    );
    assert_eq!(loaded.label_map.get("2").map(String::as_str), Some("bug"));

    fs::remove_dir_all(&root).unwrap();
}
