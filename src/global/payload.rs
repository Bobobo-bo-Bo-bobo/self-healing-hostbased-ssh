use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Message {
    pub hostname: Vec<String>,
    pub keys: Vec<Keydata>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Keydata {
    #[serde(rename = "type")]
    pub key_type: String,
    pub key: String,
    pub comment: String,
}

impl Message {
    pub fn new() -> Self {
        Message {
            hostname: Vec::new(),
            keys: Vec::new(),
        }
    }
}

impl Keydata {
    pub fn new() -> Self {
        Keydata {
            key_type: String::new(),
            key: String::new(),
            comment: String::new(),
        }
    }
}
