use crate::helpers::{
    BondTokenResponse, CurrencyResponse, Denomination, FunctionSetup, HoldersResponse,
    IssuerResponse, Phase, RedemptionAmountResponse,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20_base::msg::{
    ExecuteMsg as Cw20ExecuteMsg, InstantiateMsg as Cw20InstantiateMsg, QueryMsg as Cw20QueryMsg,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub issuer: String,
    pub basic_info: Cw20InstantiateMsg,
    pub function_setup: FunctionSetup,
    pub additional_data: String,
    pub currency: String,
    pub placeholder: String,
    pub router: String,
    pub denomination: Denomination,
    pub subscription_fee_percentage: Option<Uint128>, // fee charged by percentage, values [0 -> 10000] map to [0% -> 100%]
    pub subscription_fee: Option<Uint128>,            // fixed fee value
}

#[cw_serde]
pub enum ExecuteMsg {
    Cw20ExecuteMsg(Cw20ExecuteMsg),
    AdditionalExecuteMsg(AdditionalExecuteMsg),
}

impl ExecuteMsg {
    pub fn basic_execute_msg(self) -> Cw20ExecuteMsg {
        if let ExecuteMsg::Cw20ExecuteMsg(msg) = self {
            msg
        } else {
            panic!("Not basic execute message")
        }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BondTokenResponse)]
    Cw20QueryMsg(Cw20QueryMsg),
    #[returns(BondTokenResponse)]
    AdditionalQueryMsg(AdditionalQueryMsg),
}

impl QueryMsg {
    pub fn basic_query_msg(self) -> Cw20QueryMsg {
        if let QueryMsg::Cw20QueryMsg(msg) = self {
            msg
        } else {
            panic!("Not basic query message")
        }
    }
}

#[cw_serde]
pub enum AdditionalExecuteMsg {
    MintToInvestor {
        issuer: String,
        recipient: String,
        currency_amount: Uint128,
    },
    BurnFromHolder {
        issuer: String,
        holder: String,
    },
    Subscribe {
        subscription_amount: Uint128, // In currrency
        fee_amount: Uint128,
    },
    UpdatePhase {
        phase: Phase,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum AdditionalQueryMsg {
    #[returns(HoldersResponse)]
    GetHolders {},

    #[returns(IssuerResponse)]
    GetIssuer {},

    #[returns(CurrencyResponse)]
    GetCurrency {},

    #[returns(RedemptionAmountResponse)]
    EstimateRedempmtionAmount {},
}
