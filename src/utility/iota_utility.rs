use anyhow::{Result, bail};
use iota_streams::app::transport::tangle::client::SendOptions;
use iota_streams::core::prelude::hex;
use rand::Rng;
use iota_streams::app_channels::api::tangle::Address;
use blake2::{Blake2b, Digest};
use iota_streams::ddml::types::Bytes;

///
/// Generates a new random String of 81 Chars of A..Z and 9
///
pub fn random_seed() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    const SEED_LEN: usize = 81;
    let mut rng = rand::thread_rng();

    let seed: String = (0..SEED_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    seed
}

///
/// Generates SendOptions struct with the specified mwm and pow
///
pub fn create_send_options(min_weight_magnitude: u8, local_pow: bool) -> SendOptions{
    let mut send_opt = SendOptions::default();
    send_opt.min_weight_magnitude = min_weight_magnitude;
    send_opt.local_pow = local_pow;
    send_opt
}

pub fn hash_string(string: &str) ->  String  {
    let hash = Blake2b::digest(&string.as_bytes());
    hex::encode(&hash)
}

pub fn create_link(channel_address: &str, msg_id: &str) -> Result<Address>{
    match Address::from_str(channel_address, msg_id) {
        Ok(link) => Ok(link),
        Err(()) => bail!("Failed to create Address from {}:{}", channel_address, msg_id)
    }
}

pub fn empty_bytes() -> Bytes {
    Bytes::default()
}
