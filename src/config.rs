use std::path::PathBuf;
use std::sync::LazyLock;

use dirs::config_dir;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(alias = "command_upcase")]
    pub case: Option<Case>,
    pub enable_external_cmake_lint: bool,
    pub line_max_words: usize,
    pub format: FormatConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            case: None,
            enable_external_cmake_lint: false,
            line_max_words: 80,
            format: FormatConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Case {
    #[serde(rename = "uppercase", alias = "upcase")]
    Upper,
    #[serde(rename = "lowercase")]
    Lower,
}

#[derive(Debug, Default)]
pub struct LintSuggestion {
    pub case: Option<Case>,
    pub hint: &'static str,
}

impl LintSuggestion {
    pub fn lint_match(&self, is_uppercase: bool) -> bool {
        matches!(
            (self.case, is_uppercase),
            (Some(Case::Upper), true) | (Some(Case::Lower), false) | (None, _)
        )
    }
}

impl From<Option<Case>> for LintSuggestion {
    fn from(value: Option<Case>) -> Self {
        match value {
            Some(Case::Upper) => Self {
                case: Some(Case::Upper),
                hint: "Use uppercase command names",
            },
            Some(Case::Lower) => Self {
                case: Some(Case::Lower),
                hint: "Use lowercase command names",
            },
            None => Self {
                case: None,
                hint: "",
            },
        }
    }
}

#[derive(Default, Deserialize, PartialEq, Eq, Debug)]
pub struct FormatConfig {
    pub program: Option<String>,
    pub args: Vec<String>,
}

fn find_config_file() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;

    for file in [".neocmake.toml", ".neocmakelint.toml"] {
        let path = current_dir.join(file);
        if path.exists() {
            tracing::info!("Using project-level config file: {:?}", path);
            return Some(path);
        }
    }

    if let Some(config_dir) = config_dir() {
        for file in ["config.toml", "lint.toml"] {
            let path = config_dir.join("neocmakelsp").join(file);
            if path.exists() {
                tracing::info!("Using user-level config file: {:?}", path);
                return Some(path);
            }
        }
    }

    None
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    if let Some(path) = find_config_file() {
        if let Ok(buf) = std::fs::read_to_string(path) {
            if let Ok(config) = toml::from_str::<Config>(&buf) {
                return config;
            }
        }
    }

    Config::default()
});

pub static CMAKE_LINT: LazyLock<LintSuggestion> = LazyLock::new(|| CONFIG.case.into());
