#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use bond_token::{
    helpers::{CurrencyResponse, HoldersResponse, IssuerResponse, Phase},
    msg::{
        AdditionalExecuteMsg::{BurnFromHolder, MintToInvestor, UpdatePhase},
        AdditionalQueryMsg::{GetCurrency, GetHolders, GetIssuer},
        ExecuteMsg::AdditionalExecuteMsg,
        QueryMsg::AdditionalQueryMsg,
    },
};
use placeholder::{
    helpers::{BondValidationResponse, InvesmentRule, SubscriptionsResponse},
    msg::{
        ExecuteMsg as PlaceholderExecuteMsg, QueryMsg::SubscriptionsOf, QueryMsg::ValidateBondToken,
    },
};

use crate::{
    error::RouterErr,
    helpers::{
        Coupon, Cw20BatchBalanceQuery, Cw20BatchBalanceResponse, Cw20MintItem, Cw20TransferItem,
    },
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{RouterPlatform, OPERATORS, ROUTER_PLATFORM},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:router";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, RouterErr> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    ROUTER_PLATFORM.save(
        deps.storage,
        &RouterPlatform {
            admin: info.sender.clone(),
            placeholder: None,
            factory: None,
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
) -> Result<Response, RouterErr> {
    match msg {
        ExecuteMsg::Setup {
            placeholder,
            factory,
        } => execute::setup(deps, info, placeholder, factory),
        ExecuteMsg::SetOperators {
            operators,
            is_operators,
        } => execute::set_operators(deps, info, operators, is_operators),
        ExecuteMsg::Cw20MintBatch { cw20_mint_items } => {
            execute::cw20_mint_batch(deps, info, cw20_mint_items)
        }
        ExecuteMsg::Cw20TransferBatch {
            cw20_transfer_items,
        } => execute::cw20_transfer_batch(deps, info, cw20_transfer_items),
        ExecuteMsg::Distribute {
            bond_token,
            investment_rules,
        } => execute::distribute(deps, info, bond_token, investment_rules),
        ExecuteMsg::SendCoupon {
            bond_token,
            coupons,
        } => execute::send_coupon(deps, info, bond_token, coupons),
        ExecuteMsg::Redeem { bond_token } => execute::redeem(deps, info, bond_token),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Cw20QueryBalanceBatch {
            cw20_batch_balance_queries,
        } => to_binary(&query::balance_of_batch(deps, cw20_batch_balance_queries)?),
    }
}

pub mod execute {
    use super::*;

    pub fn setup(
        deps: DepsMut,
        info: MessageInfo,
        placeholder: String,
        factory: String,
    ) -> Result<Response, RouterErr> {
        if info.sender != ROUTER_PLATFORM.load(deps.storage)?.admin {
            return Err(RouterErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        ROUTER_PLATFORM.update(deps.storage, |mut router| -> Result<_, RouterErr> {
            router.placeholder = Some(deps.api.addr_validate(placeholder.as_str())?);
            router.factory = Some(deps.api.addr_validate(factory.as_str())?);
            Ok(router)
        })?;
        Ok(Response::new().add_attribute("action", "setup"))
    }

    pub fn set_operators(
        deps: DepsMut,
        info: MessageInfo,
        operators: Vec<String>,
        is_operators: Vec<bool>,
    ) -> Result<Response, RouterErr> {
        if info.sender != ROUTER_PLATFORM.load(deps.storage)?.admin {
            return Err(RouterErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        if operators.len() != is_operators.len() {
            return Err(RouterErr::LengthMismatch {});
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

    pub fn cw20_mint_batch(
        deps: DepsMut,
        info: MessageInfo,
        cw20_mint_items: Vec<Cw20MintItem>,
    ) -> Result<Response, RouterErr> {
        if !OPERATORS.load(deps.storage, info.sender.clone())? {
            return Err(RouterErr::NotOperator {
                account: String::from(info.sender),
            });
        }
        let mut messages: Vec<SubMsg> = vec![];
        for cw20_mint_item in cw20_mint_items.into_iter() {
            let message = Cw20ExecuteMsg::Mint {
                recipient: cw20_mint_item.recipient,
                amount: cw20_mint_item.amount,
            };
            let mint_msg = SubMsg::new(WasmMsg::Execute {
                contract_addr: cw20_mint_item.cw20_token,
                msg: to_binary(&message)?,
                funds: vec![],
            });
            messages.push(mint_msg);
        }
        Ok(Response::new()
            .add_attribute("action", "cw20_mint_batch")
            .add_submessages(messages))
    }

    pub fn cw20_transfer_batch(
        deps: DepsMut,
        info: MessageInfo,
        cw20_transfer_items: Vec<Cw20TransferItem>,
    ) -> Result<Response, RouterErr> {
        if !OPERATORS.load(deps.storage, info.sender.clone())? {
            return Err(RouterErr::NotOperator {
                account: String::from(info.sender),
            });
        }
        let mut messages: Vec<SubMsg> = vec![];
        for cw20_transfer_item in cw20_transfer_items.into_iter() {
            let message = Cw20ExecuteMsg::TransferFrom {
                owner: cw20_transfer_item.sender,
                recipient: cw20_transfer_item.recipient,
                amount: cw20_transfer_item.amount,
            };
            let transfer_msg = SubMsg::new(WasmMsg::Execute {
                contract_addr: cw20_transfer_item.cw20_token,
                msg: to_binary(&message)?,
                funds: vec![],
            });
            messages.push(transfer_msg);
        }
        Ok(Response::new()
            .add_attribute("action", "cw20_transfer_batch")
            .add_submessages(messages))
    }

    pub fn distribute(
        deps: DepsMut,
        info: MessageInfo,
        bond_token: String,
        investment_rules: Vec<InvesmentRule>,
    ) -> Result<Response, RouterErr> {
        // Query placeholder to validate this bond token
        let placeholder_addr = ROUTER_PLATFORM
            .load(deps.storage)?
            .placeholder
            .ok_or(RouterErr::ContractNotSetup {})
            .unwrap();
        let validation_response: BondValidationResponse = deps.querier.query_wasm_smart(
            placeholder_addr.to_string(),
            &ValidateBondToken {
                bond_token: bond_token.clone(),
            },
        )?;
        if !validation_response.validity {
            return Err(RouterErr::InvalidBondToken { bond_token });
        }

        // Query bond token to validate issuer
        let issuer_response: IssuerResponse = deps
            .querier
            .query_wasm_smart(bond_token.clone(), &AdditionalQueryMsg(GetIssuer {}))?;
        if issuer_response.issuer != info.sender.to_string() {
            return Err(RouterErr::NotIssuer {
                caller: info.sender.to_string(),
                bond_token,
            });
        }

        // Query placeholder to get subscriptions info
        let response: SubscriptionsResponse = deps.querier.query_wasm_smart(
            placeholder_addr.to_string(),
            &SubscriptionsOf {
                bond_token: bond_token.clone(),
            },
        )?;

        // Call bond token to update Distribution phase
        let mut messages: Vec<SubMsg> = vec![];
        messages.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: bond_token.clone(),
            msg: to_binary(&AdditionalExecuteMsg(UpdatePhase {
                phase: Phase::Distribution,
            }))?,
            funds: vec![],
        }));

        // Mint bond tokens to investors based on investment rules
        for rule in &investment_rules {
            for subscription in &response.subscriptions {
                if rule.investor == subscription.investor {
                    let invested_currency = if subscription.currency_amount > rule.currency_amount {
                        rule.currency_amount
                    } else {
                        subscription.currency_amount
                    };
                    if invested_currency > Uint128::zero() {
                        messages.push(SubMsg::new(WasmMsg::Execute {
                            contract_addr: bond_token.clone(),
                            msg: to_binary(&AdditionalExecuteMsg(MintToInvestor {
                                issuer: info.sender.to_string(),
                                recipient: rule.investor.to_string(),
                                currency_amount: invested_currency,
                            }))?,
                            funds: vec![],
                        }));
                    }
                }
            }
        }

        // Query bond token to get currency
        let currency_response: CurrencyResponse = deps
            .querier
            .query_wasm_smart(bond_token.clone(), &AdditionalQueryMsg(GetCurrency {}))?;

        // Call placeholder contract to release currency tokens
        messages.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: placeholder_addr.to_string(),
            msg: to_binary(&PlaceholderExecuteMsg::ReleaseCurrency {
                issuer: info.sender.to_string(),
                bond_token,
                currency: currency_response.currency,
                investment_rules,
            })?,
            funds: vec![],
        }));

        Ok(Response::new()
            .add_attribute("action", "distribute")
            .add_submessages(messages))
    }

    pub fn send_coupon(
        deps: DepsMut,
        info: MessageInfo,
        bond_token: String,
        coupons: Vec<Coupon>,
    ) -> Result<Response, RouterErr> {
        // Query placeholder to validate this bond token
        let placeholder_addr = ROUTER_PLATFORM
            .load(deps.storage)?
            .placeholder
            .ok_or(RouterErr::ContractNotSetup {})
            .unwrap();
        let validation_response: BondValidationResponse = deps.querier.query_wasm_smart(
            placeholder_addr.to_string(),
            &ValidateBondToken {
                bond_token: bond_token.clone(),
            },
        )?;
        if !validation_response.validity {
            return Err(RouterErr::InvalidBondToken { bond_token });
        }

        // Query bond token to validate issuer
        let issuer_response: IssuerResponse = deps
            .querier
            .query_wasm_smart(bond_token.clone(), &AdditionalQueryMsg(GetIssuer {}))?;
        if issuer_response.issuer != info.sender.to_string() {
            return Err(RouterErr::NotIssuer {
                caller: info.sender.to_string(),
                bond_token,
            });
        }

        // Query bond token to get currency
        let currency_response: CurrencyResponse = deps
            .querier
            .query_wasm_smart(bond_token.clone(), &AdditionalQueryMsg(GetCurrency {}))?;

        // Call bond token to update Coupon phase
        let mut messages: Vec<SubMsg> = vec![];
        messages.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: bond_token,
            msg: to_binary(&AdditionalExecuteMsg(UpdatePhase {
                phase: Phase::Coupon,
            }))?,
            funds: vec![],
        }));

        // Start sending
        for coupon in coupons {
            messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: currency_response.clone().currency,
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: coupon.investor,
                    amount: coupon.currency_amount,
                })?,
                funds: vec![],
            }));
        }

        Ok(Response::new()
            .add_attribute("action", "send_coupon")
            .add_submessages(messages))
    }

    pub fn redeem(
        deps: DepsMut,
        info: MessageInfo,
        bond_token: String,
    ) -> Result<Response, RouterErr> {
        // Query placeholder to validate this bond token
        let placeholder_addr = ROUTER_PLATFORM
            .load(deps.storage)?
            .placeholder
            .ok_or(RouterErr::ContractNotSetup {})
            .unwrap();
        let validation_response: BondValidationResponse = deps.querier.query_wasm_smart(
            placeholder_addr.to_string(),
            &ValidateBondToken {
                bond_token: bond_token.clone(),
            },
        )?;
        if !validation_response.validity {
            return Err(RouterErr::InvalidBondToken { bond_token });
        }

        // Query bond token to validate issuer
        let issuer_response: IssuerResponse = deps
            .querier
            .query_wasm_smart(bond_token.clone(), &AdditionalQueryMsg(GetIssuer {}))?;
        if issuer_response.issuer != info.sender.to_string() {
            return Err(RouterErr::NotIssuer {
                caller: info.sender.to_string(),
                bond_token,
            });
        }

        // Call bond token to update Redemption phase
        let mut messages: Vec<SubMsg> = vec![];
        messages.push(SubMsg::new(WasmMsg::Execute {
            contract_addr: bond_token.clone(),
            msg: to_binary(&AdditionalExecuteMsg(UpdatePhase {
                phase: Phase::Redemption,
            }))?,
            funds: vec![],
        }));

        // Query bond token to get currency
        let currency_response: CurrencyResponse = deps
            .querier
            .query_wasm_smart(bond_token.clone(), &AdditionalQueryMsg(GetCurrency {}))?;

        // Get all bond token holders
        let holders_response: HoldersResponse = deps
            .querier
            .query_wasm_smart(bond_token.clone(), &AdditionalQueryMsg(GetHolders {}))?;
        for holder in holders_response.holders {
            // Return principals to investors
            messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: currency_response.clone().currency,
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: holder.clone().account,
                    amount: holder.balance_in_currency,
                })?,
                funds: vec![],
            }));

            // Burn bond tokens from investors
            messages.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: bond_token.clone(),
                msg: to_binary(&AdditionalExecuteMsg(BurnFromHolder {
                    issuer: info.sender.to_string(),
                    holder: holder.account,
                }))?,
                funds: vec![],
            }));
        }

        Ok(Response::new()
            .add_attribute("action", "redeem")
            .add_submessages(messages))
    }
}

pub mod query {
    use super::*;

    pub fn balance_of_batch(
        deps: Deps,
        cw20_batch_balances: Vec<Cw20BatchBalanceQuery>,
    ) -> StdResult<Cw20BatchBalanceResponse> {
        let mut balances: Vec<Uint128> = vec![];
        for balance_query in cw20_batch_balances.into_iter() {
            let query_msg = Cw20QueryMsg::Balance {
                address: balance_query.cw20_holder,
            };
            let response_msg: BalanceResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: balance_query.cw20_token,
                    msg: to_binary(&query_msg)?,
                }))?;
            balances.push(response_msg.balance)
        }
        Ok(Cw20BatchBalanceResponse { balances })
    }
}
