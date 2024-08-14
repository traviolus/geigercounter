use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &str = "config";
const RADIOACTIVITY_KEY: &str = "radioactivity";

pub const SECONDS_IN_DAY: u64 = 86400;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserState {
    pub last_interaction: u64,
    pub radioactivity: u64,
}

pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
pub const RADIOACTIVITY: Map<&Addr, UserState> = Map::new(RADIOACTIVITY_KEY);
