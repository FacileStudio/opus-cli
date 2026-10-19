#[cfg(unix)]
#[test]
fn the_xdg_config_is_read_when_the_home_config_is_absent() {
    use opus_cli::config::{OpusConfig, Settings};
    use std::fs;
    use std::path::PathBuf;

    for name in [
        "OPUS_TOKEN",
        "OPUS_API_KEY",
        "OPUS_KEY",
        "OPUS_API_URL",
        "OPUS_WORKSPACE_ID",
        "OPUS_DEFAULT_PROJECT",
    ] {
        std::env::remove_var(name);
    }

    let root = std::env::temp_dir().join(format!("opus-cli-xdg-{}", std::process::id()));
    let home = root.join("home");
    let xdg = root.join("xdg");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(xdg.join("opus")).unwrap();

    std::env::set_var("HOME", &home);
    std::env::set_var("XDG_CONFIG_HOME", &xdg);

    let config_path: PathBuf = xdg.join("opus/config.yml");
    fs::write(
        &config_path,
        "api_url: http://example.test\napi_key: token\nworkspace_id: from-xdg\n",
    )
    .unwrap();

    let config = OpusConfig::load_from_path(None).expect("the XDG config should load");
    assert_eq!(config.workspace_id.as_deref(), Some("from-xdg"));
    assert_eq!(config.display_path(), config_path.display().to_string());

    let settings = Settings::resolve(None, None, false).expect("settings should resolve");
    assert_eq!(settings.api_url, "http://example.test");
    assert_eq!(settings.workspace_id, "from-xdg");
    assert_eq!(settings.api_key, "token");

    fs::remove_dir_all(&root).unwrap();
}
