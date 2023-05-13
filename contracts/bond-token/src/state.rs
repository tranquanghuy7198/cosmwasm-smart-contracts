use crate::helpers::{Denomination, FunctionSetup, Phase};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct BondToken {
    pub issuer: Addr,
    pub additional_data: String,
    pub function_setup: FunctionSetup,
    pub currency: Addr,
    pub placeholder: Addr,
    pub router: Addr,
    pub denomination: Denomination,
    pub subscription_fee_percentage: Option<Uint128>,
    pub subscription_fee: Option<Uint128>,
    pub current_phase: Phase,
}

pub const BOND_TOKEN: Item<BondToken> = Item::new("bond_token");
