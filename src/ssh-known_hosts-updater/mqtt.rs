use crate::config;

use log::{debug, error, info, warn};
use simple_error::bail;
use std::error::Error;
use std::sync::mpsc;
use std::{thread, time};

pub fn run(
    cfg: &config::Configuration,
    sender: mpsc::Sender<paho_mqtt::message::Message>,
) -> Result<(), Box<dyn Error>> {
    let one_second = time::Duration::from_secs(1);
    let conn = global::mqtt::connection_builder(&cfg.mqtt)?;
    let client = global::mqtt::client_builder(&cfg.mqtt)?;
    let cstatus: paho_mqtt::ServerResponse;

    let mut ticktock: u64 = 0;

    loop {
        let mco = conn.clone();
        cstatus = match client.connect(mco) {
            Err(e) => {
                error!(
                    "connection to MQTT broker {} failed: {}",
                    cfg.mqtt.broker, e
                );
                if ticktock > cfg.mqtt.reconnect_timeout {
                    return Err(Box::new(e));
                }
                thread::sleep(one_second);
                ticktock += 1;
                warn!(
                    "retrying to connect to MQTT broker {} - attempt {}/{}",
                    cfg.mqtt.broker, ticktock, cfg.mqtt.reconnect_timeout
                );
                continue;
            }
            Ok(v) => v,
        };
        break;
    }

    if let Some(v) = cstatus.connect_response() {
        if !v.session_present {
            info!(
                "subscribing to topic {} on {} qith QoS {}",
                cfg.mqtt.topic, cfg.mqtt.broker, cfg.mqtt.qos
            );
            if let Err(e) = client.subscribe(&cfg.mqtt.topic, cfg.mqtt.qos) {
                bail!("can't subscribe to topic {} - {}", cfg.mqtt.topic, e);
            }
        }
    } else {
        bail!("empty connect_response result from MQTT connection");
    };

    let messages = client.start_consuming();

    for msg in messages.iter() {
        match msg {
            Some(vmsg) => {
                info!("received data on {} with qos {}", vmsg.topic(), vmsg.qos());
                debug!("sending MQTT message to data handler");
                sender.send(vmsg)?;
            }
            None => {
                if !client.is_connected() {
                    warn!("connection to broker was lost, reconnecting");
                    client.reconnect()?;
                }
            }
        }
    }

    Ok(())
}
