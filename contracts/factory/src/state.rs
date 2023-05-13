use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct FactoryPlatform {
    pub admin: Addr,
    pub currency_code_id: Uint64,
    pub bond_token_code_id: Uint64,
    pub placeholder: Option<Addr>,
    pub router: Option<Addr>,
}

pub const FACTORY_PLATFORM: Item<FactoryPlatform> = Item::new("factory_platform");
pub const OPERATORS: Map<Addr, bool> = Map::new("operators");
