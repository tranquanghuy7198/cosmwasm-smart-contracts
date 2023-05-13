#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw20_base::{
    contract::{create_accounts, execute as cw20_execute, query as cw20_query},
    state::{MinterData, TokenInfo, BALANCES, TOKEN_INFO},
};

use crate::{
    error::{AdditionalError, BondTokenErr},
    execute,
    helpers::{
        CurrencyResponse, Holder, HoldersResponse, IssuerResponse, Phase, RedemptionAmountResponse,
        MAX_FEE_PERCENTAGE,
    },
    msg::{AdditionalExecuteMsg, AdditionalQueryMsg, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{BondToken, BOND_TOKEN},
};

const CONTRACT_NAME: &str = "crates.io:bond-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, BondTokenErr> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    match msg.subscription_fee_percentage {
        Some(percentage) => {
            if percentage.u128() > MAX_FEE_PERCENTAGE {
                return Err(BondTokenErr::AdditionalError(
                    AdditionalError::FeePercentageTooHigh { percentage },
                ));
            }
        }
        None => (),
    }

    // Handle special information
    BOND_TOKEN.save(
        deps.storage,
        &BondToken {
            issuer: deps.api.addr_validate(msg.issuer.as_str())?,
            additional_data: msg.additional_data,
            function_setup: msg.function_setup,
            currency: deps.api.addr_validate(msg.currency.as_str())?,
            placeholder: deps.api.addr_validate(msg.placeholder.as_str())?,
            router: deps.api.addr_validate(msg.router.as_str())?,
            denomination: msg.denomination,
            subscription_fee_percentage: msg.subscription_fee_percentage,
            subscription_fee: msg.subscription_fee,
            current_phase: Phase::Subscription,
        },
    )?;

    // Handle CW20 basic information - based on cw20_base's implementation
    msg.basic_info.validate().unwrap();

    let total_supply = match create_accounts(&mut deps, &msg.basic_info.initial_balances) {
        Ok(supply) => supply,
        Err(err) => return Err(BondTokenErr::BasicError(err)),
    };

    let mint = match msg.basic_info.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    let data = TokenInfo {
        name: msg.basic_info.name,
        symbol: msg.basic_info.symbol,
        decimals: msg.basic_info.decimals,
        total_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, BondTokenErr> {
    match msg {
        /* Additional execution functions */
        ExecuteMsg::AdditionalExecuteMsg(AdditionalExecuteMsg::MintToInvestor {
            issuer,
            recipient,
            currency_amount,
        }) => execute::mint_to_investor(deps, env, info, issuer, recipient, currency_amount),
        ExecuteMsg::AdditionalExecuteMsg(AdditionalExecuteMsg::BurnFromHolder {
            issuer,
            holder,
        }) => execute::burn_from_holder(deps, info, issuer, holder),
        ExecuteMsg::AdditionalExecuteMsg(AdditionalExecuteMsg::Subscribe {
            subscription_amount,
            fee_amount,
        }) => execute::subscribe(deps, info, subscription_amount, fee_amount),
        ExecuteMsg::AdditionalExecuteMsg(AdditionalExecuteMsg::UpdatePhase { phase }) => {
            execute::update_phase(deps, info, phase)
        }

        /* Other basic functions */
        _ => match cw20_execute(deps, env, info, msg.basic_execute_msg()) {
            Ok(response) => Ok(response),
            Err(err) => Err(BondTokenErr::BasicError(err)),
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        /* Additional queries */
        QueryMsg::AdditionalQueryMsg(AdditionalQueryMsg::GetHolders {}) => {
            to_binary(&query::get_holders(deps)?)
        }
        QueryMsg::AdditionalQueryMsg(AdditionalQueryMsg::GetIssuer {}) => {
            to_binary(&query::get_issuer(deps)?)
        }
        QueryMsg::AdditionalQueryMsg(AdditionalQueryMsg::GetCurrency {}) => {
            to_binary(&query::get_currency(deps)?)
        }
        QueryMsg::AdditionalQueryMsg(AdditionalQueryMsg::EstimateRedempmtionAmount {}) => {
            to_binary(&query::estimate_redemption_amount(deps)?)
        }

        /* Basic CW20 queries */
        _ => cw20_query(deps, env, msg.basic_query_msg()),
    }
}

pub mod query {
    use super::*;

    pub fn get_holders(deps: Deps) -> StdResult<HoldersResponse> {
        let accounts: Vec<String> = BALANCES
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|item| item.map(Into::into))
            .collect::<StdResult<_>>()?;
        let denomination = BOND_TOKEN.load(deps.storage)?.denomination;
        let mut holders: Vec<Holder> = vec![];
        for account in accounts {
            let bond_balance = BALANCES
                .may_load(deps.storage, &deps.api.addr_validate(account.as_str())?)?
                .unwrap_or_default();
            holders.push(Holder {
                account: account.clone(),
                balance_in_currency: bond_balance * denomination.currency_amount
                    / denomination.bond_amount,
            });
        }
        Ok(HoldersResponse { holders })
    }

    pub fn get_issuer(deps: Deps) -> StdResult<IssuerResponse> {
        let issuer = BOND_TOKEN.load(deps.storage)?.issuer.to_string();
        Ok(IssuerResponse { issuer })
    }

    pub fn get_currency(deps: Deps) -> StdResult<CurrencyResponse> {
        let currency = BOND_TOKEN.load(deps.storage)?.currency.to_string();
        Ok(CurrencyResponse { currency })
    }

    pub fn estimate_redemption_amount(deps: Deps) -> StdResult<RedemptionAmountResponse> {
        let total_supply = TOKEN_INFO.load(deps.storage)?.total_supply;
        let denomination = BOND_TOKEN.load(deps.storage)?.denomination;
        Ok(RedemptionAmountResponse {
            redemption_amount: total_supply * denomination.currency_amount
                / denomination.bond_amount,
        })
    }
}
