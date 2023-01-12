use crate::config;
use crate::constants;

use log::{info, warn};
use simple_error::bail;
use std::error::Error;

pub fn send(cfg: &config::Configuration, hostlist: Vec<String>) -> Result<(), Box<dyn Error>> {
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
    global::mqtt::connect(&cfg.mqtt, &mqtt_client, &mqtt_connection)?;
    info!(
        "connected to MQTT broker {} with client ID {}",
        cfg.mqtt.broker, cfg.mqtt.client_id
    );

    if !mqtt_client.is_connected() {
        warn!(
            "connection to MQTT broker {} lost, reconnecting",
            cfg.mqtt.broker
        );
        if let Err(e) = global::mqtt::reconnect(&cfg.mqtt, &mqtt_client) {
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

    info!("disconnecting from MQTT broker {}", cfg.mqtt.broker);
    if let Err(e) = global::mqtt::disconnect(&mqtt_client) {
        warn!("diconnect from MQTT broker failed: {}", e);
    };

    Ok(())
}
