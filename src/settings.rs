use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

pub const MIN_UPDATE_MS: u64 = 100;
pub const MAX_UPDATE_MS: u64 = 60_000;
pub const UPDATE_STEP_MS: u64 = 100;
pub const TIMEFRAME_LABELS: [&str; 7] =
    ["1 min", "5 min", "15 min", "30 min", "1 h", "4 h", "1 dia"];

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Timeframe {
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "30m")]
    M30,
    #[default]
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "4h")]
    H4,
    #[serde(rename = "1d")]
    D1,
}

impl Timeframe {
    pub const fn as_binance(self) -> &'static str {
        match self {
            Self::M1 => "1m",
            Self::M5 => "5m",
            Self::M15 => "15m",
            Self::M30 => "30m",
            Self::H1 => "1h",
            Self::H4 => "4h",
            Self::D1 => "1d",
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Self::M1 => 0,
            Self::M5 => 1,
            Self::M15 => 2,
            Self::M30 => 3,
            Self::H1 => 4,
            Self::H4 => 5,
            Self::D1 => 6,
        }
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => Self::M1,
            1 => Self::M5,
            2 => Self::M15,
            3 => Self::M30,
            5 => Self::H4,
            6 => Self::D1,
            _ => Self::H1,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
    pub update_interval_ms: u64,
    pub timeframe: Timeframe,
    pub show_change_percent: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            update_interval_ms: 1_000,
            timeframe: Timeframe::H1,
            show_change_percent: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        fs::read_to_string(config_path())
            .ok()
            .and_then(|content| serde_json::from_str::<Self>(&content).ok())
            .filter(|settings| {
                (MIN_UPDATE_MS..=MAX_UPDATE_MS).contains(&settings.update_interval_ms)
            })
            .unwrap_or_default()
    }

    pub fn save(self) -> io::Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temporary = path.with_extension("json.tmp");
        let content = serde_json::to_vec_pretty(&self).map_err(io::Error::other)?;
        fs::write(&temporary, content)?;
        fs::rename(temporary, path)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DraftSettings {
    pub update_interval_ms: u64,
    pub timeframe_index: usize,
    pub show_change_percent: bool,
}

impl From<Settings> for DraftSettings {
    fn from(settings: Settings) -> Self {
        Self {
            update_interval_ms: settings.update_interval_ms,
            timeframe_index: settings.timeframe.index(),
            show_change_percent: settings.show_change_percent,
        }
    }
}

impl DraftSettings {
    pub fn parse(self) -> Result<Settings, String> {
        if !(MIN_UPDATE_MS..=MAX_UPDATE_MS).contains(&self.update_interval_ms) {
            return Err(format!(
                "Use um intervalo entre {MIN_UPDATE_MS} e {MAX_UPDATE_MS} ms."
            ));
        }

        Ok(Settings {
            update_interval_ms: self.update_interval_ms,
            timeframe: Timeframe::from_index(self.timeframe_index),
            show_change_percent: self.show_change_percent,
        })
    }
}

fn config_path() -> PathBuf {
    if let Some(base) = env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(base)
            .join("com.github.drwesleyadv.Ticker")
            .join("settings.json");
    }

    let base = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".config")
        .join("com.github.drwesleyadv.Ticker")
        .join("settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeframe_roundtrip_uses_supported_indices() {
        for index in 0..TIMEFRAME_LABELS.len() {
            assert_eq!(Timeframe::from_index(index).index(), index);
        }
    }

    #[test]
    fn draft_validates_update_interval() {
        let mut draft = DraftSettings::from(Settings::default());
        draft.update_interval_ms = 99;
        assert!(draft.parse().is_err());

        draft.update_interval_ms = 250;
        assert_eq!(draft.parse().unwrap().update_interval_ms, 250);
    }
}
