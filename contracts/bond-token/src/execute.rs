use cosmwasm_std::{
    to_binary, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20_base::{
    contract,
    msg::ExecuteMsg as Cw20ExecuteMsg,
    state::{BALANCES, TOKEN_INFO},
};
use placeholder::msg::ExecuteMsg as PlaceholderExecuteMsg;

use crate::{
    error::{AdditionalError, BondTokenErr},
    helpers::{Phase, MAX_FEE_PERCENTAGE},
    state::BOND_TOKEN,
};

/* Overrided CW20 functions */

pub fn burn_from_holder(
    deps: DepsMut,
    info: MessageInfo,
    issuer: String,
    holder: String,
) -> Result<Response, BondTokenErr> {
    if !BOND_TOKEN.load(deps.storage)?.function_setup.burn {
        return Err(BondTokenErr::AdditionalError(
            AdditionalError::FunctionNotSupported {
                function: String::from("burn_from_holder"),
            },
        ));
    }

    // Only router can call this function to burn
    if BOND_TOKEN.load(deps.storage)?.router != info.sender {
        return Err(BondTokenErr::AdditionalError(AdditionalError::NotRouter {
            caller: info.sender.to_string(),
        }));
    }

    // Validate issuer
    if BOND_TOKEN.load(deps.storage)?.issuer.to_string() != issuer {
        return Err(BondTokenErr::AdditionalError(AdditionalError::NotIssuer {
            caller: info.sender.to_string(),
        }));
    }

    // Burn all bond token from this holder
    let holder_addr = deps.api.addr_validate(holder.as_str())?;
    BALANCES.update(deps.storage, &holder_addr, |_| -> StdResult<_> {
        Ok(Uint128::zero())
    })?;

    // Reduce total_supply
    let balance = BALANCES.load(deps.storage, &holder_addr)?;
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(balance)?;
        Ok(info)
    })?;

    Ok(Response::new()
        .add_attribute("action", "burn_from_holder")
        .add_attribute("from", holder))
}

pub fn mint_to_investor(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    issuer: String,
    recipient: String,
    currency_amount: Uint128,
) -> Result<Response, BondTokenErr> {
    if !BOND_TOKEN
        .load(deps.storage)?
        .function_setup
        .mint_to_investor
    {
        return Err(BondTokenErr::AdditionalError(
            AdditionalError::FunctionNotSupported {
                function: String::from("mint_to_investor"),
            },
        ));
    }

    // Validate issuer
    if BOND_TOKEN.load(deps.storage)?.issuer.to_string() != issuer {
        return Err(BondTokenErr::AdditionalError(AdditionalError::NotIssuer {
            caller: issuer,
        }));
    }

    // Calculate bond amount
    let denomination = BOND_TOKEN.load(deps.storage)?.denomination;
    let bond_amount = currency_amount * denomination.bond_amount / denomination.currency_amount;

    match contract::execute_mint(deps, env, info, recipient, bond_amount) {
        Ok(response) => Ok(response),
        Err(err) => Err(BondTokenErr::BasicError(err)),
    }
}

/* Additional functions */

pub fn subscribe(
    deps: DepsMut,
    info: MessageInfo,
    subscription_amount: Uint128,
    fee_amount: Uint128,
) -> Result<Response, BondTokenErr> {
    let bond_token_platform = BOND_TOKEN.load(deps.storage)?;
    if !bond_token_platform.function_setup.subscribe {
        return Err(BondTokenErr::AdditionalError(
            AdditionalError::FunctionNotSupported {
                function: String::from("subscribe"),
            },
        ));
    }

    // Can only subscribe in the Subscription phase
    if bond_token_platform.current_phase != Phase::Subscription {
        return Err(BondTokenErr::AdditionalError(
            AdditionalError::ActionNotAllowed {
                action: String::from("subscription"),
            },
        ));
    }

    let currency_amount = subscription_amount + fee_amount;

    // Calculate fee, charging by percentage is more prioritized than charging by fixed value
    let mut subscription_fee = Uint128::zero();
    match BOND_TOKEN.load(deps.storage)?.subscription_fee_percentage {
        Some(percentage) => {
            subscription_fee = currency_amount * percentage / Uint128::from(MAX_FEE_PERCENTAGE)
        }
        None => match BOND_TOKEN.load(deps.storage)?.subscription_fee {
            Some(fee) => {
                if currency_amount <= fee {
                    return Err(BondTokenErr::AdditionalError(
                        AdditionalError::InsufficientSubscriptionAmount {
                            amount: currency_amount,
                        },
                    ));
                }
                subscription_fee = fee;
            }
            None => subscription_fee = fee_amount,
        },
    }

    // Transfer currency from investor's wallet to system placeholder
    let mut messages: Vec<SubMsg> = vec![];
    messages.push(SubMsg::new(WasmMsg::Execute {
        contract_addr: bond_token_platform.currency.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: info.sender.to_string(),
            recipient: bond_token_platform.placeholder.to_string(),
            amount: currency_amount,
        })?,
        funds: vec![],
    }));

    // Register investor's subscription in placeholder
    messages.push(SubMsg::new(WasmMsg::Execute {
        contract_addr: BOND_TOKEN.load(deps.storage)?.placeholder.to_string(),
        msg: to_binary(&PlaceholderExecuteMsg::RegisterSubscription {
            investor: info.sender.to_string(),
            currency: bond_token_platform.currency.to_string(),
            subscription_amount: currency_amount - subscription_fee,
            fee_amount: subscription_fee,
        })?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_attribute("action", "subscribe")
        .add_submessages(messages))
}

pub fn update_phase(
    deps: DepsMut,
    info: MessageInfo,
    phase: Phase,
) -> Result<Response, BondTokenErr> {
    // Only router can update phase
    if info.sender != BOND_TOKEN.load(deps.storage)?.router {
        return Err(BondTokenErr::AdditionalError(
            AdditionalError::NotRouterPlaceholder {
                caller: info.sender.to_string(),
            },
        ));
    }

    BOND_TOKEN.update(deps.storage, |mut bond_token| -> Result<_, BondTokenErr> {
        // Can only increase at most one phase
        match bond_token.current_phase {
            Phase::Subscription => {
                if phase != Phase::Distribution {
                    return Err(BondTokenErr::AdditionalError(AdditionalError::InvalidPhase));
                }
            }
            Phase::Distribution => {
                if phase != Phase::Coupon {
                    return Err(BondTokenErr::AdditionalError(AdditionalError::InvalidPhase));
                }
            }
            Phase::Coupon => {
                if phase != Phase::Coupon && phase != Phase::Redemption {
                    return Err(BondTokenErr::AdditionalError(AdditionalError::InvalidPhase));
                }
            }
            Phase::Redemption => {
                return Err(BondTokenErr::AdditionalError(AdditionalError::InvalidPhase));
            }
        }
        if bond_token.current_phase != phase {
            bond_token.current_phase = phase;
        }
        Ok(bond_token)
    })?;

    Ok(Response::new().add_attribute("action", "update_phase"))
}
