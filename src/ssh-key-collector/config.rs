use log::debug;
use serde::Deserialize;
use simple_error::bail;
use std::error::Error;
use std::fs;
use std::path::Path;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct Configuration {
    pub mqtt: global::mqtt::MQTT,
    #[serde(rename = "ssh-keys")]
    #[serde(default)]
    pub ssh_keys: SSHKeys,
    #[serde(skip)]
    pub ssh_directory: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SSHKeys {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub hostname: Vec<String>,
    pub comment: Option<String>,
}

pub fn parse_config_file(f: &str) -> Result<Configuration, Box<dyn Error>> {
    let raw = fs::read_to_string(f)?;
    let mut parsed: Configuration = serde_yaml::from_str(raw.as_str())?;

    validate(&parsed)?;

    parsed.mqtt.topic = parsed.mqtt.topic.trim_end_matches('/').to_string();
    parsed.mqtt.topic = format!(
        "{}/{}",
        parsed.mqtt.topic,
        gethostname::gethostname().into_string().unwrap()
    );

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

pub fn fill_missing_fields(cfg: &mut Configuration) -> Result<(), Box<dyn Error>> {
    if cfg.ssh_keys.files.is_empty() {
        cfg.ssh_keys.files = find_pub_keys(&cfg.ssh_directory)?;
    }

    if cfg.ssh_keys.hostname.is_empty() {
        cfg.ssh_keys
            .hostname
            .push(gethostname::gethostname().into_string().unwrap());
    }
    Ok(())
}

fn find_pub_keys(dir: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut result: Vec<String> = Vec::new();

    if dir.is_empty() {
        bail!("empty directory");
    }

    debug!("looking for SSH public keys in {}", dir);
    for entry in (fs::read_dir(Path::new(dir))?).flatten() {
        let p = entry.path();
        debug!("processing {:?}", p);
        if p.is_file() {
            let fname = match p.to_str() {
                Some(v) => v,
                None => {
                    continue;
                }
            };

            let bname = match p.file_name() {
                Some(v) => v.to_str().unwrap(),
                None => {
                    continue;
                }
            };
            if bname.starts_with("ssh_host_") && bname.ends_with("_key.pub") {
                result.push(fname.to_string());
            }
        }
    }
    Ok(result)
}
