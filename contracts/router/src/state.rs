use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct RouterPlatform {
    pub admin: Addr,
    pub placeholder: Option<Addr>,
    pub factory: Option<Addr>,
}

pub const ROUTER_PLATFORM: Item<RouterPlatform> = Item::new("router_platform");
pub const OPERATORS: Map<Addr, bool> = Map::new("operators");
