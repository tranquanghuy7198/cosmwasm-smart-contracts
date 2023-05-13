#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, SubMsg,
    Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg::Transfer;

use crate::{
    error::PlaceholderErr,
    helpers::{
        BondValidationResponse, InvesmentRule, Subscription, SubscriptionResponse,
        SubscriptionsResponse,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{PlaceholderPlatform, OPERATORS, PLACEHOLDER_PLATFORM, SUBSCRIPTIONS, SYSTEM_FEE},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:placeholder";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, PlaceholderErr> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    PLACEHOLDER_PLATFORM.save(
        deps.storage,
        &PlaceholderPlatform {
            admin: info.sender.clone(),
            factory: None,
            router: None,
            currencies: vec![],
            bond_tokens: vec![],
        },
    )?;
    OPERATORS.save(deps.storage, info.sender.clone(), &true)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, PlaceholderErr> {
    match msg {
        ExecuteMsg::Setup { factory, router } => execute::setup(deps, info, factory, router),
        ExecuteMsg::SetOperators {
            operators,
            is_operators,
        } => execute::set_operators(deps, info, operators, is_operators),
        // ExecuteMsg::RegisterCurrencies { currencies, states } => {
        //     execute::register_currency(deps, info, currencies, states)
        // }
        ExecuteMsg::RegisterBondToken { bond_token } => {
            execute::register_bond_token(deps, info, bond_token)
        }
        ExecuteMsg::RegisterSubscription {
            investor,
            currency,
            subscription_amount,
            fee_amount,
        } => execute::register_subcription(
            deps,
            info,
            investor,
            currency,
            subscription_amount,
            fee_amount,
        ),
        ExecuteMsg::ReleaseCurrency {
            issuer,
            bond_token,
            currency,
            investment_rules,
        } => execute::release_currency(deps, info, issuer, bond_token, currency, investment_rules),
        ExecuteMsg::WithdrawSystemFee { recipient } => {
            execute::withdraw_system_fee(deps, info, recipient)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SubscriptionsOf { bond_token } => {
            to_binary(&query::get_subscriptions(deps, bond_token)?)
        }
        QueryMsg::ValidateBondToken { bond_token } => {
            to_binary(&query::validate_bond_token(deps, bond_token)?)
        }
    }
}

pub mod execute {
    use super::*;

    pub fn setup(
        deps: DepsMut,
        info: MessageInfo,
        factory: String,
        router: String,
    ) -> Result<Response, PlaceholderErr> {
        if info.sender != PLACEHOLDER_PLATFORM.load(deps.storage)?.admin {
            return Err(PlaceholderErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        PLACEHOLDER_PLATFORM.update(
            deps.storage,
            |mut placeholder| -> Result<_, PlaceholderErr> {
                placeholder.factory = Some(deps.api.addr_validate(factory.as_str())?);
                placeholder.router = Some(deps.api.addr_validate(router.as_str())?);
                Ok(placeholder)
            },
        )?;
        Ok(Response::new().add_attribute("action", "setup"))
    }

    pub fn set_operators(
        deps: DepsMut,
        info: MessageInfo,
        operators: Vec<String>,
        is_operators: Vec<bool>,
    ) -> Result<Response, PlaceholderErr> {
        if info.sender != PLACEHOLDER_PLATFORM.load(deps.storage)?.admin {
            return Err(PlaceholderErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        if operators.len() != is_operators.len() {
            return Err(PlaceholderErr::LengthMismatch {});
        }
        for (i, new_operator) in operators.iter().enumerate() {
            OPERATORS.save(
                deps.storage,
                deps.api.addr_validate(new_operator.as_str())?,
                is_operators.get(i).unwrap(),
            )?;
        }
        Ok(Response::new().add_attribute("action", "set_operators"))
    }

    // pub fn register_currency(
    //     deps: DepsMut,
    //     info: MessageInfo,
    //     currencies: Vec<String>,
    //     states: Vec<bool>,
    // ) -> Result<Response, PlaceholderErr> {
    //     // Only System Addresses can register currencies
    //     if !OPERATORS.load(deps.storage, info.sender.clone())? {
    //         return Err(PlaceholderErr::NotOperator {
    //             account: info.sender.to_string(),
    //         });
    //     }

    //     if currencies.len() != states.len() {
    //         return Err(PlaceholderErr::LengthMismatch {});
    //     }

    //     PLACEHOLDER_PLATFORM.update(deps.storage, |mut platform| -> Result<_, PlaceholderErr> {
    //         for (index, currency) in currencies.iter().enumerate() {
    //             if states.swap_remove(index) {
    //                 //
    //             }
    //         }
    //         Ok(platform)
    //     })?;

    //     Ok(Response::new().add_attribute("action", "register_currency"))
    // }

    pub fn register_bond_token(
        deps: DepsMut,
        info: MessageInfo,
        bond_token: String,
    ) -> Result<Response, PlaceholderErr> {
        // Only System Addresses can register bond tokens
        if !OPERATORS.load(deps.storage, info.sender.clone())? {
            return Err(PlaceholderErr::NotOperator {
                account: info.sender.to_string(),
            });
        }

        PLACEHOLDER_PLATFORM.update(deps.storage, |mut platform| -> Result<_, PlaceholderErr> {
            platform
                .bond_tokens
                .push(deps.api.addr_validate(bond_token.as_str())?);
            Ok(platform)
        })?;

        Ok(Response::new().add_attribute("action", "register_bond_token"))
    }

    pub fn register_subcription(
        deps: DepsMut,
        info: MessageInfo,
        investor: String,
        currency: String,
        subscription_amount: Uint128,
        fee_amount: Uint128,
    ) -> Result<Response, PlaceholderErr> {
        // Only bond token can call this function to register investor's subscription
        if !PLACEHOLDER_PLATFORM
            .load(deps.storage)?
            .bond_tokens
            .into_iter()
            .any(|bond_token| bond_token == info.sender)
        {
            return Err(PlaceholderErr::NotBondToken {
                account: info.sender.to_string(),
            });
        }

        // Update fee
        let currency_address = deps.api.addr_validate(currency.as_str())?;
        SYSTEM_FEE.update(deps.storage, currency_address, |fee| -> StdResult<_> {
            Ok(fee.unwrap_or_default() + fee_amount)
        })?;

        // Register investor's subscription
        SUBSCRIPTIONS.update(
            deps.storage,
            info.sender,
            |subs| -> Result<_, PlaceholderErr> {
                match subs {
                    Some(mut subscriptions) => {
                        // If this is the new investor
                        if !subscriptions
                            .clone()
                            .into_iter()
                            .any(|subscription| subscription.investor.to_string() == investor)
                        {
                            subscriptions.push(Subscription {
                                investor: deps.api.addr_validate(&investor)?,
                                currency_amount: subscription_amount,
                            });
                        } else {
                            // This investor already invested before
                            for subscription in subscriptions.iter_mut() {
                                if subscription.investor.to_string() == investor {
                                    subscription.currency_amount += subscription_amount;
                                }
                            }
                        }
                        Ok(subscriptions)
                    }
                    None => Ok(vec![Subscription {
                        investor: deps.api.addr_validate(&investor)?,
                        currency_amount: subscription_amount,
                    }]),
                }
            },
        )?;

        Ok(Response::new().add_attribute("action", "register_subcription"))
    }

    // Release currency from placeholder to issuer and return investors their excess
    pub fn release_currency(
        deps: DepsMut,
        info: MessageInfo,
        issuer: String,
        bond_token: String,
        currency: String,
        investment_rules: Vec<InvesmentRule>,
    ) -> Result<Response, PlaceholderErr> {
        // Only router can call this function to release currency
        if PLACEHOLDER_PLATFORM
            .load(deps.storage)?
            .router
            .ok_or(PlaceholderErr::ContractNotSetup {})
            .unwrap()
            != info.sender
        {
            return Err(PlaceholderErr::NotRouter {
                account: info.sender.to_string(),
            });
        }

        let mut messages: Vec<SubMsg> = vec![];
        let mut invested_currency = Uint128::zero();
        let subscriptions =
            SUBSCRIPTIONS.load(deps.storage, deps.api.addr_validate(bond_token.as_str())?)?;
        for subscription in &subscriptions {
            let mut max_allowed_currency_subscription = Uint128::zero();
            for rule in &investment_rules {
                if subscription.investor.to_string() == rule.investor {
                    max_allowed_currency_subscription = rule.currency_amount;
                    break;
                }
            }
            if subscription.currency_amount > max_allowed_currency_subscription {
                invested_currency += max_allowed_currency_subscription;

                // Return excess to investors
                messages.push(SubMsg::new(WasmMsg::Execute {
                    contract_addr: currency.clone(),
                    msg: to_binary(&Transfer {
                        recipient: subscription.investor.to_string(),
                        amount: subscription.currency_amount - max_allowed_currency_subscription,
                    })?,
                    funds: vec![],
                }));
            } else {
                invested_currency += subscription.currency_amount;
            }
        }

        // Transfer invested currency to issuer
        messages.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: currency,
            msg: to_binary(&Transfer {
                recipient: issuer,
                amount: invested_currency,
            })?,
            funds: vec![],
        }));

        Ok(Response::new()
            .add_attribute("action", "release_currency")
            .add_submessages(messages))
    }

    pub fn withdraw_system_fee(
        deps: DepsMut,
        info: MessageInfo,
        recipient: String,
    ) -> Result<Response, PlaceholderErr> {
        // Only admin can withdraw system fee
        if PLACEHOLDER_PLATFORM.load(deps.storage)?.admin != info.sender {
            return Err(PlaceholderErr::NotAdmin {
                account: info.sender.to_string(),
            });
        }

        // Transfer all fees to the recipient
        let currencies: Vec<String> = SYSTEM_FEE
            .keys(deps.storage, None, None, Order::Ascending)
            .map(|item| item.map(Into::into))
            .collect::<StdResult<_>>()?;
        let mut withdraw_messages: Vec<SubMsg> = vec![];
        for currency in currencies {
            withdraw_messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: currency.clone(),
                msg: to_binary(&Transfer {
                    recipient: recipient.clone(),
                    amount: SYSTEM_FEE
                        .load(deps.storage, deps.api.addr_validate(currency.as_str())?)?,
                })?,
                funds: vec![],
            }));
        }

        Ok(Response::new()
            .add_attribute("action", "withdraw_system_fee")
            .add_submessages(withdraw_messages))
    }
}

pub mod query {
    use super::*;

    pub fn get_subscriptions(deps: Deps, bond_token: String) -> StdResult<SubscriptionsResponse> {
        let subscriptions = SUBSCRIPTIONS
            .load(deps.storage, deps.api.addr_validate(bond_token.as_str())?)?
            .iter()
            .map(|s| SubscriptionResponse {
                investor: s.investor.to_string(),
                currency_amount: s.currency_amount,
            })
            .collect();
        Ok(SubscriptionsResponse { subscriptions })
    }

    pub fn validate_bond_token(
        deps: Deps,
        bond_token: String,
    ) -> StdResult<BondValidationResponse> {
        let validity = PLACEHOLDER_PLATFORM
            .load(deps.storage)?
            .bond_tokens
            .iter()
            .any(|token| token.to_string() == bond_token);
        Ok(BondValidationResponse { validity })
    }
}
