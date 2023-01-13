use crate::constants;

use log::{error, warn};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;
use std::{thread, time};

#[derive(Clone, Debug, Deserialize)]
pub struct MQTT {
    pub broker: String,
    #[serde(default)]
    pub ca_cert: String,
    #[serde(default)]
    pub clean_session: bool,
    #[serde(default = "mqtt_default_client_id")]
    pub client_id: String,
    #[serde(default)]
    pub insecure_ssl: bool,
    pub password: String,
    #[serde(default)]
    pub qos: i32,
    #[serde(default = "mqtt_default_reconnect_timeout")]
    pub reconnect_timeout: u64,
    #[serde(default = "mqtt_default_timeout")]
    pub timeout: u64,
    pub topic: String,
    pub user: String,
}

fn mqtt_default_timeout() -> u64 {
    constants::DEFAULT_MQTT_TIMEOUT
}

fn mqtt_default_reconnect_timeout() -> u64 {
    constants::DEFAULT_MQTT_RECONNECT_TIMEOUT
}

fn mqtt_default_client_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(constants::MAXIMAL_CLIENT_ID_LENGTH)
        .map(char::from)
        .collect()
}

pub fn connection_builder(
    cfg: &MQTT,
) -> Result<paho_mqtt::connect_options::ConnectOptions, Box<dyn Error>> {
    let mut sslopts = paho_mqtt::ssl_options::SslOptionsBuilder::new();
    if cfg.broker.starts_with("ssl://") || cfg.broker.starts_with("tls://") {
        if !cfg.ca_cert.is_empty() {
            sslopts.trust_store(&cfg.ca_cert)?;
        }
        if cfg.insecure_ssl {
            sslopts.enable_server_cert_auth(false);
            sslopts.verify(false);
        } else {
            sslopts.enable_server_cert_auth(true);
            sslopts.verify(true);
        }
    }

    let client_opt = paho_mqtt::connect_options::ConnectOptionsBuilder::new()
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(cfg.timeout))
        .clean_session(cfg.clean_session)
        .connect_timeout(Duration::from_secs(cfg.timeout))
        .user_name(&cfg.user)
        .password(&cfg.password)
        .retry_interval(Duration::from_secs(1))
        .ssl_options(sslopts.finalize())
        .finalize();

    Ok(client_opt)
}

pub fn client_builder(cfg: &MQTT) -> Result<paho_mqtt::client::Client, Box<dyn Error>> {
    let client_opts = paho_mqtt::CreateOptionsBuilder::new()
        .client_id(&cfg.client_id)
        .server_uri(&cfg.broker)
        .persistence(None)
        .finalize();

    let client = paho_mqtt::client::Client::new(client_opts)?;
    Ok(client)
}

pub fn connect(
    cfg: &MQTT,
    client: &paho_mqtt::client::Client,
    option: &paho_mqtt::connect_options::ConnectOptions,
) -> Result<paho_mqtt::server_response::ServerResponse, Box<dyn Error>> {
    let mut ticktock: u64 = 0;
    let cstatus: paho_mqtt::server_response::ServerResponse;
    let one_second = time::Duration::from_secs(1);

    loop {
        let mco = option.clone();
        cstatus = match client.connect(mco) {
            Err(e) => {
                error!("connection to MQTT broker {} failed: {}", cfg.broker, e);
                if cfg.reconnect_timeout != 0 && ticktock > cfg.reconnect_timeout {
                    return Err(Box::new(e));
                }
                thread::sleep(one_second);
                ticktock += 1;
                if cfg.reconnect_timeout != 0 {
                    warn!(
                        "retrying to connect to MQTT broker {} - attempt {}/{}",
                        cfg.broker, ticktock, cfg.reconnect_timeout
                    );
                } else {
                    warn!(
                        "retrying to connect to MQTT broker {} - attempt {}",
                        cfg.broker, ticktock
                    );
                }
                continue;
            }
            Ok(v) => v,
        };
        break;
    }
    Ok(cstatus)
}

pub fn reconnect(
    cfg: &MQTT,
    client: &paho_mqtt::client::Client,
) -> Result<paho_mqtt::server_response::ServerResponse, Box<dyn Error>> {
    let mut ticktock: u64 = 0;
    let cstatus: paho_mqtt::server_response::ServerResponse;
    let one_second = time::Duration::from_secs(1);

    loop {
        cstatus = match client.reconnect() {
            Err(e) => {
                error!("reconnect to MQTT broker {} failed: {}", cfg.broker, e);
                if cfg.reconnect_timeout != 0 && ticktock > cfg.reconnect_timeout {
                    return Err(Box::new(e));
                }
                thread::sleep(one_second);
                ticktock += 1;
                if cfg.reconnect_timeout != 0 {
                    warn!(
                        "retrying to reconnect to MQTT broker {} - attempt {}/{}",
                        cfg.broker, ticktock, cfg.reconnect_timeout
                    );
                } else {
                    warn!(
                        "retrying to reconnect to MQTT broker {} - attempt {}",
                        cfg.broker, ticktock
                    );
                }
                continue;
            }
            Ok(v) => v,
        };
        break;
    }
    Ok(cstatus)
}

pub fn disconnect(client: &paho_mqtt::client::Client) -> Result<(), Box<dyn Error>> {
    if let Err(e) = client.disconnect(None) {
        return Err(Box::new(e));
    };
    Ok(())
}
