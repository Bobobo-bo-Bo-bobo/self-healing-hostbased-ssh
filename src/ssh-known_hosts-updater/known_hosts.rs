use crate::config;
use crate::constants;

use log::{debug, error, info};
use mktemp::Temp;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::sync::mpsc;

pub fn update(
    cfg: &config::Configuration,
    receiver: mpsc::Receiver<paho_mqtt::message::Message>,
) -> Result<(), Box<dyn Error>> {
    let mut ssh_known_hosts: HashMap<String, global::payload::Message> = HashMap::new();
    let tempdir = match env::var("TMPDIR") {
        Ok(v) => v,
        Err(_) => constants::DEFAULT_TMPDIR.to_string(),
    };

    loop {
        let mqtt_msg = receiver.recv()?;

        debug!("MQTT message received for processing");

        let mut update = false;
        let topic = mqtt_msg.topic();
        let payload = mqtt_msg.payload().to_vec();
        let mut spl_tpc: Vec<&str> = topic.split('/').collect();

        let key = match spl_tpc.pop() {
            Some(v) => v.to_string(),
            None => {
                panic!("BUG: empty topic - shouldn't happen");
            }
        };

        debug!("parsing MQTT message");
        if payload.is_empty() {
            info!("empty message received for {}, removing data from map", key);
            ssh_known_hosts.remove(&key);
            update = true
        } else {
            let msg = match parse_data(payload) {
                Ok(v) => v,
                Err(e) => {
                    error!("can't parse message payload: {}", e);
                    continue;
                }
            };

            debug!("processing MQTT message for {}", key);
            if msg.keys.is_empty() {
                debug!("key list is empty, removing {} from map", key);
                ssh_known_hosts.remove(&key);
                update = true;
            } else if let Some(oldvalue) = ssh_known_hosts.get(&key) {
                debug!("processing non-empty data for {}", key);
                if *oldvalue != msg {
                    debug!("key information for {} changed, updating data", key);
                    ssh_known_hosts.insert(key, msg);
                    update = true;
                } else {
                    info!(
                        "key information for {} has not changed, skipping update of {}",
                        key, cfg.ssh.known_hosts_file,
                    );
                }
            } else {
                debug!("SSH key data not found for {}, inserting data", key);
                ssh_known_hosts.insert(key, msg);
                update = true;
            }
        }
        if update {
            if let Err(e) =
                update_ssh_known_hosts_file(&cfg.ssh.known_hosts_file, &ssh_known_hosts, &tempdir)
            {
                error!("can't update {}: {}", cfg.ssh.known_hosts_file, e);
            }
        }
    }
}

fn parse_data(raw: Vec<u8>) -> Result<global::payload::Message, Box<dyn Error>> {
    let raw_str = String::from_utf8(raw)?;
    let parsed = serde_json::from_str(&raw_str)?;
    Ok(parsed)
}

fn update_ssh_known_hosts_file(
    file: &str,
    data: &HashMap<String, global::payload::Message>,
    tmpdir: &str,
) -> Result<(), Box<dyn Error>> {
    let mut keys: Vec<String> = Vec::new();
    let tempfile = Temp::new_file_in(tmpdir)?;
    if let Some(tempfile_name) = tempfile.to_str() {
        for value in data.values() {
            for key in value.keys.iter() {
                keys.push(format!(
                    "{} {} {} {}",
                    value.hostname.join(","),
                    key.key_type,
                    key.key,
                    key.comment
                ));
            }
        }

        info!("writing key data to {}", tempfile_name);
        let mut content = keys.join("\n");
        content.push('\n');

        fs::write(tempfile_name, content)?;

        info!("replacing {} with new content from {}", file, tempfile_name);
        fs::rename(tempfile_name, file)?;
    }
    Ok(())
}
