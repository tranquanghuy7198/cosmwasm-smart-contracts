use cosmwasm_std::{Addr, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AssetVault {
    pub admin: Addr,
}

pub const ASSET_VAULT: Item<AssetVault> = Item::new("asset_vault");
pub const OPERATORS: Map<Addr, bool> = Map::new("operators");
