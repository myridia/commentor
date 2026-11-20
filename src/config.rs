use homedir::my_home;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    InvalidConfig(toml::de::Error),
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::InvalidConfig(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub last_file: Option<String>,
    #[serde(default = "default_remember_last_file")]
    pub remember_last_file: bool,
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,
    #[serde(default)]
    pub comment_wrap: bool,
    #[serde(default = "default_show_link_colors")]
    pub show_link_colors: bool,
    #[serde(default = "default_space_lines")]
    pub space_lines: usize,
    #[serde(default = "default_source_font_size")]
    pub source_font_size: f32,
    #[serde(default = "default_comment_font_size")]
    pub comment_font_size: f32,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,
    #[serde(default = "default_highlight")]
    pub highlight: bool,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,
}

fn default_remember_last_file() -> bool {
    true
}

fn default_show_line_numbers() -> bool {
    true
}

fn default_show_link_colors() -> bool {
    true
}

fn default_space_lines() -> usize {
    2
}

fn default_source_font_size() -> f32 {
    13.0
}

fn default_comment_font_size() -> f32 {
    13.0
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_color_scheme() -> String {
    "blue".to_string()
}

fn default_highlight() -> bool {
    true
}

fn default_language() -> String {
    "auto".to_string()
}

fn default_syntax_theme() -> String {
    "default".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_file: None,
            remember_last_file: default_remember_last_file(),
            show_line_numbers: default_show_line_numbers(),
            comment_wrap: false,
            show_link_colors: default_show_link_colors(),
            space_lines: default_space_lines(),
            source_font_size: default_source_font_size(),
            comment_font_size: default_comment_font_size(),
            theme: default_theme(),
            color_scheme: default_color_scheme(),
            highlight: default_highlight(),
            language: default_language(),
            syntax_theme: default_syntax_theme(),
        }
    }
}

pub fn get_config() -> AppConfig {
    match load_or_initialize() {
        Ok(config) => config,
        Err(error) => {
            match error {
                ConfigError::IoError(error) => {
                    eprintln!("An error occurred while loading the config: {error}");
                }
                ConfigError::InvalidConfig(error) => {
                    eprintln!("An error occurred while parsing the config:");
                    eprintln!("{error}");
                }
            }
            AppConfig::default()
        }
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), ConfigError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let toml = toml::to_string_pretty(config).unwrap();
    fs::write(path, toml)?;
    Ok(())
}

pub fn config_path() -> Result<PathBuf, ConfigError> {
    let home = my_home().ok().flatten().ok_or_else(|| {
        ConfigError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "Cannot determine home directory",
        ))
    })?;
    Ok(home.join(".commentor").join("config.toml"))
}

fn load_or_initialize() -> Result<AppConfig, ConfigError> {
    let path = config_path()?;
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        return Ok(config);
    }

    let config = AppConfig::default();
    save_config(&config)?;
    Ok(config)
}
