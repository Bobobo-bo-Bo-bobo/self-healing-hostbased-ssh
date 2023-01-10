pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const DEFAULT_MQTT_TIMEOUT: u64 = 15;
pub const DEFAULT_MQTT_RECONNECT_TIMEOUT: u64 = 300;
pub const MAXIMAL_CLIENT_ID_LENGTH: usize = 23;
pub const DEFAULT_SSH_DIRECTORY: &str = "/etc/ssh";
