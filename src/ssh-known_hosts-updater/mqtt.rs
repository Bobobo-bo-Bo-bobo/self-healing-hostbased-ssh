use crate::config;

use log::{debug, info, warn};
use simple_error::bail;
use std::error::Error;
use std::sync::mpsc;

pub fn run(
    cfg: &config::Configuration,
    sender: mpsc::Sender<paho_mqtt::message::Message>,
) -> Result<(), Box<dyn Error>> {
    let conn = global::mqtt::connection_builder(&cfg.mqtt)?;
    let client = global::mqtt::client_builder(&cfg.mqtt)?;
    let cstatus = global::mqtt::connect(&cfg.mqtt, &client, &conn)?;

    info!(
        "connected to MQTT broker {} with client ID {}",
        cfg.mqtt.broker, cfg.mqtt.client_id
    );

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
                    global::mqtt::reconnect(&cfg.mqtt, &client)?;
                }
            }
        }
    }

    Ok(())
}
