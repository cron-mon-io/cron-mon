pub mod create_alert_config;
pub mod delete_alert_config;
pub mod fetch_alert_configs;
pub mod test_alert_config;
pub mod update_alert_config;

use serde::Deserialize;

pub use create_alert_config::CreateAlertConfigService;
pub use delete_alert_config::DeleteAlertConfigService;
pub use fetch_alert_configs::FetchAlertConfigs;
pub use test_alert_config::TestAlertConfigService;
pub use update_alert_config::UpdateAlertConfigService;

#[derive(Deserialize)]
pub struct AlertConfigData {
    pub name: String,
    pub active: bool,
    pub on_late: bool,
    pub on_error: bool,
    #[serde(rename = "type")]
    pub type_: serde_json::Value,
}
