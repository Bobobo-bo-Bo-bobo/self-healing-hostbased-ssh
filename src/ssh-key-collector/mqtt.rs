use crate::config;

use log::{info, warn};
use simple_error::bail;
use std::error::Error;

pub fn send(
    cfg: &config::Configuration,
    data: &global::payload::Message,
) -> Result<(), Box<dyn Error>> {
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

    let payload = match serde_json::to_string(data) {
        Ok(v) => v,
        Err(e) => {
            bail!("can't convert message to JSON: {}", e);
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

    info!(
        "sending data to topic {} on MQTT broker {}",
        cfg.mqtt.topic, cfg.mqtt.broker
    );
    let msg = paho_mqtt::message::Message::new_retained(&cfg.mqtt.topic, payload, cfg.mqtt.qos);
    if let Err(e) = mqtt_client.publish(msg) {
        bail!("sending message to MQTT broker failed - {}", e);
    }

    info!("disconnecting from MQTT broker {}", cfg.mqtt.broker);
    if let Err(e) = global::mqtt::disconnect(&mqtt_client) {
        warn!("diconnect from MQTT broker failed: {}", e);
    };

    Ok(())
}
