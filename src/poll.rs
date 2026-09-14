use crate::settings::Settings;

pub fn update_period_ms(settings: Settings) -> u64 {
    settings.update_interval_ms
}
