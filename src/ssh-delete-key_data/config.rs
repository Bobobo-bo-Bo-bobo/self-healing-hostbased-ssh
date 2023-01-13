use serde::Deserialize;
use simple_error::bail;
use std::error::Error;
use std::fs;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct Configuration {
    pub mqtt: global::mqtt::MQTT,
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

    if cfg.mqtt.topic.is_empty() || cfg.mqtt.topic.contains('+') || cfg.mqtt.topic.contains('#') {
        bail!("invalid MQTT topic, wildcards are not allowed in publishing topic")
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

    Ok(())
}

fn validate_url(s: &str) -> Result<(), Box<dyn Error>> {
    let _parsed = Url::parse(s)?;
    Ok(())
}

pub fn validate_hostname(h: &str) -> Result<(), Box<dyn Error>> {
    if h.contains('/') || h.contains('+') || h.contains('#') {
        bail!("invalid hostname");
    }
    Ok(())
}

