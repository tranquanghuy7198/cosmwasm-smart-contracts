use bond_token::{
    helpers::{Denomination, FunctionSetup},
    msg::InstantiateMsg as BondTokenInstantiateMsg,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20Coin, MinterResponse};
use cw20_base::msg::{InstantiateMarketingInfo, InstantiateMsg as Cw20InstantiateMsg};

use crate::{
    error::FactoryErr,
    helpers::ContractInfo,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{FactoryPlatform, FACTORY_PLATFORM, OPERATORS},
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, FactoryErr> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    FACTORY_PLATFORM.save(
        deps.storage,
        &FactoryPlatform {
            admin: info.sender.clone(),
            currency_code_id: msg.currency_code_id,
            bond_token_code_id: msg.bond_token_code_id,
            placeholder: None,
            router: None,
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
) -> Result<Response, FactoryErr> {
    match msg {
        ExecuteMsg::Setup {
            placeholder,
            router,
        } => execute::setup(deps, info, placeholder, router),
        ExecuteMsg::SetOperators {
            operators,
            is_operators,
        } => execute::set_operators(deps, info, operators, is_operators),
        ExecuteMsg::InstantiateCurrency {
            name,
            symbol,
            decimals,
            initial_balances,
            mint,
            marketing,
        } => execute::instantiate_currency(
            deps,
            info,
            name,
            symbol,
            decimals,
            initial_balances,
            mint,
            marketing,
        ),
        ExecuteMsg::InstantiateBondToken {
            issuer,
            name,
            symbol,
            decimals,
            initial_balances,
            function_setup,
            additional_data,
            currency,
            denomination,
            subscription_fee_percentage,
            subscription_fee,
        } => execute::instantiate_bond_token(
            deps,
            info,
            issuer,
            name,
            symbol,
            decimals,
            initial_balances,
            function_setup,
            additional_data,
            currency,
            denomination,
            subscription_fee_percentage,
            subscription_fee,
        ),
        ExecuteMsg::InstantiateBatch { contract_infos } => {
            execute::instantiate_batch(deps, info, contract_infos)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

pub mod execute {
    use super::*;

    pub fn setup(
        deps: DepsMut,
        info: MessageInfo,
        placeholder: String,
        router: String,
    ) -> Result<Response, FactoryErr> {
        if info.sender != FACTORY_PLATFORM.load(deps.storage)?.admin {
            return Err(FactoryErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        FACTORY_PLATFORM.update(deps.storage, |mut factory| -> Result<_, FactoryErr> {
            factory.placeholder = Some(deps.api.addr_validate(placeholder.as_str())?);
            factory.router = Some(deps.api.addr_validate(router.as_str())?);
            Ok(factory)
        })?;
        Ok(Response::new().add_attribute("action", "setup"))
    }

    pub fn set_operators(
        deps: DepsMut,
        info: MessageInfo,
        operators: Vec<String>,
        is_operators: Vec<bool>,
    ) -> Result<Response, FactoryErr> {
        if info.sender != FACTORY_PLATFORM.load(deps.storage)?.admin {
            return Err(FactoryErr::NotAdmin {
                account: info.sender.into(),
            });
        }
        if operators.len() != is_operators.len() {
            return Err(FactoryErr::LengthMismatch {});
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

    pub fn instantiate_currency(
        deps: DepsMut,
        info: MessageInfo,
        name: String,
        symbol: String,
        decimals: u8,
        initial_balances: Vec<Cw20Coin>,
        mint: Option<MinterResponse>,
        marketing: Option<InstantiateMarketingInfo>,
    ) -> Result<Response, FactoryErr> {
        // Only System Addresses can instantiate bond tokens
        if !OPERATORS.load(deps.storage, info.sender.clone())? {
            return Err(FactoryErr::NotOperator {
                account: info.sender.to_string(),
            });
        }

        // Instantiate
        let instantiate_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(info.sender.to_string()),
            code_id: FACTORY_PLATFORM.load(deps.storage)?.currency_code_id.u64(),
            msg: to_binary(&Cw20InstantiateMsg {
                name,
                symbol,
                decimals,
                initial_balances,
                mint,
                marketing,
            })?,
            funds: vec![],
            label: String::from("currency"),
        });

        Ok(Response::new()
            .add_attribute("action", "instantiate_currency")
            .add_message(instantiate_msg))
    }

    pub fn instantiate_bond_token(
        deps: DepsMut,
        info: MessageInfo,
        issuer: String,
        name: String,
        symbol: String,
        decimals: u8,
        initial_balances: Vec<Cw20Coin>,
        function_setup: FunctionSetup,
        additional_data: String,
        currency: String,
        denomination: Denomination,
        subscription_fee_percentage: Option<Uint128>,
        subscription_fee: Option<Uint128>,
    ) -> Result<Response, FactoryErr> {
        // Only System Addresses can instantiate bond tokens
        if !OPERATORS.load(deps.storage, info.sender.clone())? {
            return Err(FactoryErr::NotOperator {
                account: info.sender.to_string(),
            });
        }

        // Instantiate
        let factory = FACTORY_PLATFORM.load(deps.storage)?;
        let instantiate_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: Some(info.sender.to_string()),
            code_id: factory.bond_token_code_id.u64(),
            msg: to_binary(&BondTokenInstantiateMsg {
                issuer,
                basic_info: Cw20InstantiateMsg {
                    name,
                    symbol,
                    decimals,
                    initial_balances,
                    mint: Some(MinterResponse {
                        minter: factory
                            .router
                            .clone()
                            .ok_or(FactoryErr::ContractNotSetup {})
                            .unwrap()
                            .to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                function_setup,
                additional_data,
                currency,
                placeholder: factory
                    .placeholder
                    .ok_or(FactoryErr::ContractNotSetup {})
                    .unwrap()
                    .to_string(),
                router: factory
                    .router
                    .ok_or(FactoryErr::ContractNotSetup {})
                    .unwrap()
                    .to_string(),
                denomination,
                subscription_fee_percentage,
                subscription_fee,
            })?,
            funds: vec![],
            label: String::from("bond_token"),
        });

        Ok(Response::new()
            .add_attribute("action", "instantiate_bond_token")
            .add_message(instantiate_msg))
    }

    pub fn instantiate_batch(
        deps: DepsMut,
        info: MessageInfo,
        contract_infos: Vec<ContractInfo>,
    ) -> Result<Response, FactoryErr> {
        if !OPERATORS.load(deps.storage, info.sender.clone())? {
            return Err(FactoryErr::NotOperator {
                account: info.sender.to_string(),
            });
        }
        let mut messages: Vec<CosmosMsg> = vec![];
        for contract_info in contract_infos.into_iter() {
            messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(info.sender.to_string()),
                code_id: contract_info.code_id.u64(),
                msg: contract_info.instantiate_msg,
                funds: contract_info.funds,
                label: contract_info.label,
            }));
        }
        Ok(Response::new()
            .add_attribute("action", "instantiate_batch")
            .add_messages(messages))
    }
}
