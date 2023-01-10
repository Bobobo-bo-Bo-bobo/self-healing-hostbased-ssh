use crate::config;

use log::info;
use std::error::Error;
use std::fs;

pub fn read_key_files(
    cfg: &config::Configuration,
) -> Result<global::payload::Message, Box<dyn Error>> {
    let mut result = global::payload::Message::new();

    result.hostname = cfg.ssh_keys.hostname.clone();
    for f in cfg.ssh_keys.files.iter() {
        info!("reading {}", f);
        let raw = match fs::read_to_string(f) {
            Ok(v) => v,
            Err(e) => return Err(Box::new(e)),
        };

        let mut parsed_key = parse_key_data(&raw);
        if !cfg.ssh_keys.comment.is_empty() {
            parsed_key.comment = cfg.ssh_keys.comment.clone();
        }

        result.keys.push(parsed_key);
    }

    Ok(result)
}

fn parse_key_data(raw: &str) -> global::payload::Keydata {
    let splitted: Vec<&str> = raw.splitn(3, ' ').collect();
    global::payload::Keydata {
        key_type: splitted[0].to_string(),
        key: splitted[1].to_string(),
        comment: splitted[2].to_string(),
    }
}
