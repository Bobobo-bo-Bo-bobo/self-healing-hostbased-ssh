use crate::config;
use crate::constants;

use log::{error, info, warn};
use simple_error::bail;
use std::error::Error;
use std::{thread, time};

pub fn send(cfg: &config::Configuration, hostlist: Vec<String>) -> Result<(), Box<dyn Error>> {
    let one_second = time::Duration::from_secs(1);

    let mqtt_connection = match global::mqtt::connection_builder(&cfg.mqtt) {
        Ok(v) => v,
        Err(e) => {
            bail!("can't build MQTT connection structure: {}", e);
        }
    };

    let mqtt_client = match global::mqtt::client_builder(&cfg.mqtt) {
        Ok(v) => v,
        Err(e) => {
            bail!("can't build MQTT client structure: {}", e);
        }
    };

    info!("connecting to MQTT broker {}", cfg.mqtt.broker);
    let mut tickticktick: u64 = 0;

    loop {
        let mco = mqtt_connection.clone();
        if let Err(e) = mqtt_client.connect(mco) {
            error!(
                "connection to MQTT broker {} failed: {}",
                cfg.mqtt.broker, e
            );
            if tickticktick > cfg.mqtt.reconnect_timeout {
                return Err(Box::new(e));
            }
            thread::sleep(one_second);
            tickticktick += 1;
            warn!(
                "retrying to connect to MQTT broker {} - attempt {}/{}",
                cfg.mqtt.broker, tickticktick, cfg.mqtt.reconnect_timeout
            );
        } else {
            info!(
                "connected to MQTT broker {} with client ID {}",
                cfg.mqtt.broker, cfg.mqtt.client_id
            );
            break;
        }
    }

    if !mqtt_client.is_connected() {
        warn!(
            "connection to MQTT broker {} lost, reconnecting",
            cfg.mqtt.broker
        );
        if let Err(e) = mqtt_client.reconnect() {
            bail!(
                "reconnection to MQTT broker {} failed - {}",
                cfg.mqtt.broker,
                e
            );
        }
    }

    for host in hostlist {
        let topic = format!("{}/{}", cfg.mqtt.topic, host);
        let mqtt_msg = paho_mqtt::message::MessageBuilder::new()
            .topic(&topic)
            .payload(constants::EMPTY_MESSAGE)
            .qos(cfg.mqtt.qos)
            .retained(true)
            .finalize();

        info!(
            "sending data to topic {} on MQTT broker {}",
            topic, cfg.mqtt.broker
        );
        if let Err(e) = mqtt_client.publish(mqtt_msg) {
            bail!("sending message to MQTT broker failed - {}", e);
        }
    }

    Ok(())
}
