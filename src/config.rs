//! Validated configuration. Defaults, then a TOML file, then CLI flags.

use crate::cli::Cli;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub output: OutputSettings,
    pub selection: SelectionSettings,
    pub report: ReportSettings,
    pub limits: Limits,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            output: OutputSettings::default(),
            selection: SelectionSettings::default(),
            report: ReportSettings::default(),
            limits: Limits::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OutputSettings {
    pub headers: bool,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self { headers: true }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SelectionSettings {
    pub respect_ignore_files: bool,
    pub include_hidden: bool,
    pub exclude: Vec<String>,
}

impl Default for SelectionSettings {
    fn default() -> Self {
        Self {
            respect_ignore_files: true,
            include_hidden: false,
            exclude: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Tokenizer {
    #[serde(rename = "o200k_base")]
    O200kBase,
    #[serde(rename = "cl100k_base")]
    Cl100kBase,
    #[serde(rename = "off")]
    Off,
}

impl Tokenizer {
    pub fn label(self) -> &'static str {
        match self {
            Self::O200kBase => "o200k_base",
            Self::Cl100kBase => "cl100k_base",
            Self::Off => "off",
        }
    }
}

impl FromStr for Tokenizer {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self> {
        match text {
            "o200k_base" => Ok(Self::O200kBase),
            "cl100k_base" => Ok(Self::Cl100kBase),
            "off" => Ok(Self::Off),
            other => bail!("unknown tokenizer '{other}'; choose o200k_base, cl100k_base or off"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReportSettings {
    pub tokenizer: Tokenizer,
}

impl Default for ReportSettings {
    fn default() -> Self {
        Self {
            tokenizer: Tokenizer::O200kBase,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    pub max_files: usize,
    pub max_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_files: 5_000,
            max_bytes: 50_000_000,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.limits.max_files == 0 {
            bail!("limits.max_files must be greater than zero");
        }
        if self.limits.max_bytes == 0 {
            bail!("limits.max_bytes must be greater than zero");
        }
        // Selection patterns are validated separately when constructing the matcher.
        Ok(())
    }
}

/// Deliberately use ~/.config on macOS as well as Linux, rather than
/// macOS's Application Support directory, for an easy-to-edit CLI config.
fn default_config_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        dirs::config_dir().map(|directory| directory.join("cpc").join("config.toml"))
    }
    #[cfg(not(windows))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")));
        base.map(|directory| directory.join("cpc").join("config.toml"))
    }
}

pub fn load_config(explicit_path: Option<&Path>) -> Result<Settings> {
    let Some(path) = explicit_path
        .map(Path::to_path_buf)
        .or_else(default_config_path)
    else {
        return Ok(Settings::default());
    };
    if explicit_path.is_none() && !path.exists() {
        return Ok(Settings::default());
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("cannot read configuration {}", path.display()))?;
    let settings: Settings = toml::from_str(&text)
        .with_context(|| format!("invalid configuration {}", path.display()))?;
    settings.validate()?;
    Ok(settings)
}

pub fn resolve_settings(mut settings: Settings, args: &Cli) -> Result<Settings> {
    if args.no_headers {
        settings.output.headers = false;
    }
    if args.hidden {
        settings.selection.include_hidden = true;
    }
    if args.no_ignore {
        settings.selection.respect_ignore_files = false;
    }
    if args.no_tokens {
        settings.report.tokenizer = Tokenizer::Off;
    }
    if let Some(name) = &args.tokenizer {
        settings.report.tokenizer = name.parse()?;
    }
    settings.validate()?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe_and_tokenised() {
        let settings = Settings::default();
        assert!(settings.output.headers);
        assert!(settings.selection.respect_ignore_files);
        assert!(!settings.selection.include_hidden);
        assert_eq!(settings.report.tokenizer, Tokenizer::O200kBase);
    }

    #[test]
    fn partial_config_preserves_defaults() {
        let settings: Settings = toml::from_str("[output]\nheaders = false\n").unwrap();
        assert!(!settings.output.headers);
        assert_eq!(settings.limits.max_files, 5000);
    }

    #[test]
    fn reject_typos_in_config() {
        let result = toml::from_str::<Settings>("[selection]\ninclude_hiden = true\n");
        assert!(result.is_err());
    }

    #[test]
    fn reject_unknown_tokenizer() {
        assert!(toml::from_str::<Settings>("[report]\ntokenizer = 'imaginary'\n").is_err());
    }
}
