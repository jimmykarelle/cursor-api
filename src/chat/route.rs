mod logs;
pub use logs::{handle_logs, handle_logs_post};
mod health;
pub use health::{handle_health, handle_root};
mod token;
pub use token::{
    handle_basic_calibration, handle_get_checksum, handle_get_tokeninfo, handle_tokeninfo_page,
    handle_update_tokeninfo, handle_update_tokeninfo_post,
};
mod profile;
pub use profile::get_user_info;
mod config;
pub use config::{
    handle_about, handle_config_page, handle_env_example, handle_readme, handle_static,
};
mod api;
pub use api::handle_api_page;
