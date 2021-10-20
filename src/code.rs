use crate::prelude::*;
use base64::{display::Base64Display, URL_SAFE_NO_PAD};

pub fn encode(id: &Uuid) -> Base64Display {
    Base64Display::with_config(id.as_bytes(), URL_SAFE_NO_PAD)
}

pub fn decode(str: &str) -> Option<Uuid> {
    if str.len() != 22 {
        return None;
    }

    let mut out = [0; 16];
    base64::decode_config_slice(str, URL_SAFE_NO_PAD, &mut out).ok()?;
    Some(Uuid::from_bytes(out))
}
