use crate::constants;

use serde::Deserialize;
use simple_error::bail;
use std::error::Error;
use std::fs;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct Configuration {
    pub mqtt: global::mqtt::MQTT,
    #[serde(default)]
    pub ssh: Ssh,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Ssh {
    #[serde(default = "default_ssh_known_hosts_file")]
    pub known_hosts_file: String,
}

fn default_ssh_known_hosts_file() -> String {
    constants::DEFAULT_SSH_KNOWN_HOSTS_FILE.to_string()
}

pub fn parse_config_file(f: &str) -> Result<Configuration, Box<dyn Error>> {
    let raw = fs::read_to_string(f)?;
    let mut parsed: Configuration = serde_yaml::from_str(raw.as_str())?;

    validate(&parsed)?;

    parsed.mqtt.topic = parsed.mqtt.topic.trim_end_matches('/').to_string();

    Ok(parsed)
}

fn validate(cfg: &Configuration) -> Result<(), Box<dyn Error>> {
    if cfg.mqtt.qos > 2 || cfg.mqtt.qos < 0 {
        bail!("invalid MQTT QoS setting");
    }

    if cfg.mqtt.topic.is_empty() || (!cfg.mqtt.topic.contains('+') && !cfg.mqtt.topic.contains('#'))
    {
        bail!("invalid MQTT topic, wildcards must be present in subscribed topic")
    }

    if cfg.mqtt.timeout == 0 {
        bail!("invalid MQTT timeout");
    }

    if cfg.mqtt.reconnect_timeout == 0 {
        bail!("invalid MQTT reconnect timeout");
    }

    if let Err(e) = validate_url(&cfg.mqtt.broker) {
        bail!("invalid MQTT broker url: {}", e);
    }

    if cfg.ssh.known_hosts_file.is_empty() {
        bail!("empty value for ssh known_hosts file");
    }

    Ok(())
}

fn validate_url(s: &str) -> Result<(), Box<dyn Error>> {
    let _parsed = Url::parse(s)?;
    Ok(())
}
