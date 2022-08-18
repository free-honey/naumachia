#![allow(unused)]
use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct Genesis {
    active_slots_coefficient: f32,
    update_quorum: u32,
    max_lovelace_supply: String,
    network_magic: u32,
    epoch_length: u32,
    system_start: u32,
    slots_per_kes_period: u32,
    slot_length: u32,
    max_kes_evolutions: u32,
    security_param: u32,
}
