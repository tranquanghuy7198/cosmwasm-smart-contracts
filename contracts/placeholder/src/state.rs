use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::helpers::Subscription;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PlaceholderPlatform {
    pub admin: Addr,
    pub factory: Option<Addr>,
    pub router: Option<Addr>,
    pub currencies: Vec<Addr>,
    pub bond_tokens: Vec<Addr>,
}

pub const PLACEHOLDER_PLATFORM: Item<PlaceholderPlatform> = Item::new("placeholder_platform");
pub const OPERATORS: Map<Addr, bool> = Map::new("operators");
pub const SUBSCRIPTIONS: Map<Addr, Vec<Subscription>> = Map::new("subscriptions"); // maps from a bond token to its investors' subcriptions
pub const SYSTEM_FEE: Map<Addr, Uint128> = Map::new("system_fee");
