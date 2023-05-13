#[cfg(test)]
mod tests {
    use bond_token::{
        helpers::{Denomination, FunctionSetup, RedemptionAmountResponse},
        msg::{
            AdditionalExecuteMsg, AdditionalQueryMsg, ExecuteMsg as BondTokenExecuteMsg,
            QueryMsg as BondTokenQueryMsg,
        },
    };
    use cosmwasm_std::{coins, Addr, Uint128, Uint64};
    use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg};
    use cw20_base::msg::QueryMsg as Cw20BaseQueryMsg;
    use cw_multi_test::{App, AppBuilder, ContractWrapper, Executor};
    use factory::msg::ExecuteMsg as FactoryExecuteMsg;
    use placeholder::{helpers::InvesmentRule, msg::ExecuteMsg as PlaceholderExecuteMsg};
    use router::{helpers::Coupon, msg::ExecuteMsg as RouterExecuteMsg};

    const NATIVE_DENOM: &str = "flavor";

    const ADMIN: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4a";
    const ISSUER: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4b";
    const INVESTOR_1: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4c";
    const INVESTOR_2: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4d";
    const OPERATOR: &str = "cosmos10w2pwzxaacsj508ma5ruz5wnhn83tld73shr4e"; // System accounts

    const SUBSCRIPTION_FEE_PERCENTAGE: u128 = 5000; // 50% fee

    fn mock_blockchain() -> App {
        AppBuilder::new().build(|router, _, storage| {
            for account in vec![ADMIN, ISSUER, INVESTOR_1, INVESTOR_2, OPERATOR].iter() {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked(*account),
                        coins(1000, NATIVE_DENOM),
                    )
                    .unwrap();
            }
        })
    }

    fn proper_instantiate() -> (App, Addr, Addr, Addr, u64, u64) {
        let mut blockchain = mock_blockchain();
        let bond_token_id = blockchain.store_code(Box::new(ContractWrapper::new(
            bond_token::contract::execute,
            bond_token::contract::instantiate,
            bond_token::contract::query,
        )));
        let currency_id = blockchain.store_code(Box::new(ContractWrapper::new(
            currency::execute,
            currency::instantiate,
            currency::query,
        )));
        let factory_id = blockchain.store_code(Box::new(ContractWrapper::new(
            factory::contract::execute,
            factory::contract::instantiate,
            factory::contract::query,
        )));
        let placeholder_id = blockchain.store_code(Box::new(ContractWrapper::new(
            placeholder::contract::execute,
            placeholder::contract::instantiate,
            placeholder::contract::query,
        )));
        let router_id = blockchain.store_code(Box::new(ContractWrapper::new(
            router::contract::execute,
            router::contract::instantiate,
            router::contract::query,
        )));

        let factory_address = blockchain
            .instantiate_contract(
                factory_id,
                Addr::unchecked(ADMIN),
                &factory::msg::InstantiateMsg {
                    currency_code_id: Uint64::from(currency_id),
                    bond_token_code_id: Uint64::from(bond_token_id),
                },
                &[],
                "factory",
                Some(String::from(ADMIN)),
            )
            .unwrap();

        let placeholder_address = blockchain
            .instantiate_contract(
                placeholder_id,
                Addr::unchecked(ADMIN),
                &placeholder::msg::InstantiateMsg {},
                &[],
                "placeholder",
                Some(String::from(ADMIN)),
            )
            .unwrap();

        let router_address = blockchain
            .instantiate_contract(
                router_id,
                Addr::unchecked(ADMIN),
                &router::msg::InstantiateMsg {},
                &[],
                "router",
                Some(String::from(ADMIN)),
            )
            .unwrap();

        (
            blockchain,
            factory_address,
            placeholder_address,
            router_address,
            currency_id,
            bond_token_id,
        )
    }

    mod factory_test {
        use super::*;

        #[test]
        fn factory_test() {
            let (
                mut blockchain,
                factory_address,
                placeholder_address,
                router_address,
                currency_code_id,
                bond_token_code_id,
            ) = proper_instantiate();

            /* ================= Admin creates a new currency ================= */
            let mut currency_address = Addr::unchecked("");
            let mut transaction = blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    factory_address.clone(),
                    &FactoryExecuteMsg::InstantiateCurrency {
                        name: String::from("Tether USDT"),
                        symbol: String::from("USDT"),
                        decimals: 6,
                        initial_balances: vec![
                            Cw20Coin {
                                address: Addr::unchecked(INVESTOR_1).to_string(),
                                amount: Uint128::from(1000_u128),
                            },
                            Cw20Coin {
                                address: Addr::unchecked(INVESTOR_2).to_string(),
                                amount: Uint128::from(2000_u128),
                            },
                            Cw20Coin {
                                address: Addr::unchecked(ISSUER).to_string(),
                                amount: Uint128::from(500_u128),
                            },
                        ],
                        mint: None,
                        marketing: None,
                    },
                    &[],
                )
                .unwrap();
            // Extract currency address from the events
            for event in transaction.events {
                if event
                    .attributes
                    .iter()
                    .any(|attr| attr.key == "code_id" && attr.value == currency_code_id.to_string())
                {
                    for attribute in event.attributes {
                        if attribute.key == String::from("_contract_addr") {
                            currency_address = Addr::unchecked(attribute.value);
                            break;
                        }
                    }
                    break;
                }
            }

            /* ================= Set up factory, placeholder and router ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    factory_address.clone(),
                    &FactoryExecuteMsg::Setup {
                        placeholder: placeholder_address.to_string(),
                        router: router_address.to_string(),
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    placeholder_address.clone(),
                    &PlaceholderExecuteMsg::Setup {
                        factory: factory_address.to_string(),
                        router: router_address.to_string(),
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    router_address.clone(),
                    &RouterExecuteMsg::Setup {
                        placeholder: placeholder_address.to_string(),
                        factory: factory_address.to_string(),
                    },
                    &[],
                )
                .unwrap();

            /* ================= Set operators ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    factory_address.clone(),
                    &FactoryExecuteMsg::SetOperators {
                        operators: vec![Addr::unchecked(OPERATOR).to_string()],
                        is_operators: vec![true],
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    placeholder_address.clone(),
                    &FactoryExecuteMsg::SetOperators {
                        operators: vec![Addr::unchecked(OPERATOR).to_string()],
                        is_operators: vec![true],
                    },
                    &[],
                )
                .unwrap();

            /* ================= Issuer creates bond token ================= */
            let mut bond_token_address = Addr::unchecked("");
            transaction = blockchain
                .execute_contract(
                    Addr::unchecked(OPERATOR),
                    factory_address.clone(),
                    &FactoryExecuteMsg::InstantiateBondToken {
                        issuer: String::from(ISSUER),
                        name: String::from("Bond Token"),
                        symbol: String::from("BOND-TOKEN"),
                        decimals: 18,
                        initial_balances: vec![],
                        function_setup: FunctionSetup {
                            transfer: true,
                            burn: true,
                            mint_to_investor: true,
                            subscribe: true,
                        },
                        additional_data: String::from("no additional data"),
                        currency: currency_address.to_string(),
                        denomination: Denomination {
                            currency_amount: Uint128::from(3_u128),
                            bond_amount: Uint128::from(2_u128),
                        },
                        subscription_fee_percentage: Some(Uint128::from(
                            SUBSCRIPTION_FEE_PERCENTAGE,
                        )),
                        subscription_fee: None,
                    },
                    &[],
                )
                .unwrap();
            // Extract bond token address from the events
            for event in transaction.events {
                if event.attributes.iter().any(|attr| {
                    attr.key == "code_id" && attr.value == bond_token_code_id.to_string()
                }) {
                    for attribute in event.attributes {
                        if attribute.key == String::from("_contract_addr") {
                            bond_token_address = Addr::unchecked(attribute.value);
                            break;
                        }
                    }
                    break;
                }
            }

            /* ================= Register bond token to placeholder ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(OPERATOR),
                    placeholder_address.clone(),
                    &PlaceholderExecuteMsg::RegisterBondToken {
                        bond_token: bond_token_address.to_string(),
                    },
                    &[],
                )
                .unwrap();

            /* ================= Investor subscribes to bond token ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(INVESTOR_1),
                    currency_address.clone(),
                    &Cw20ExecuteMsg::IncreaseAllowance {
                        spender: bond_token_address.to_string(),
                        amount: Uint128::from(600_u128),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(INVESTOR_1),
                    bond_token_address.clone(),
                    &BondTokenExecuteMsg::AdditionalExecuteMsg(AdditionalExecuteMsg::Subscribe {
                        subscription_amount: Uint128::from(600_u128),
                        fee_amount: Uint128::zero(),
                    }),
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(INVESTOR_2),
                    currency_address.clone(),
                    &Cw20ExecuteMsg::IncreaseAllowance {
                        spender: bond_token_address.to_string(),
                        amount: Uint128::from(1134_u128),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(INVESTOR_2),
                    bond_token_address.clone(),
                    &BondTokenExecuteMsg::AdditionalExecuteMsg(AdditionalExecuteMsg::Subscribe {
                        subscription_amount: Uint128::from(1134_u128),
                        fee_amount: Uint128::zero(),
                    }),
                    &[],
                )
                .unwrap();
            let mut placeholder_balance: BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: placeholder_address.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(placeholder_balance.balance.u128(), 1734_u128);

            /* ================= Issuer distributes bond tokens ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(ISSUER),
                    router_address.clone(),
                    &RouterExecuteMsg::Distribute {
                        bond_token: bond_token_address.to_string(),
                        investment_rules: vec![
                            InvesmentRule {
                                investor: Addr::unchecked(INVESTOR_1).to_string(),
                                currency_amount: Uint128::from(270_u128),
                            },
                            InvesmentRule {
                                investor: Addr::unchecked(INVESTOR_2).to_string(),
                                currency_amount: Uint128::from(1000_u128),
                            },
                        ],
                    },
                    &[],
                )
                .unwrap();
            placeholder_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: placeholder_address.to_string(),
                    },
                )
                .unwrap();
            let mut investor1_currency_balance: BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_1).to_string(),
                    },
                )
                .unwrap();
            let mut investor1_bond_balance: BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    bond_token_address.clone(),
                    &BondTokenQueryMsg::Cw20QueryMsg(Cw20BaseQueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_1).to_string(),
                    }),
                )
                .unwrap();
            let mut investor2_currency_balance: BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_2).to_string(),
                    },
                )
                .unwrap();
            let mut investor2_bond_balance: BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    bond_token_address.clone(),
                    &BondTokenQueryMsg::Cw20QueryMsg(Cw20BaseQueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_2).to_string(),
                    }),
                )
                .unwrap();
            let mut issuer_balance: BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(ISSUER).to_string(),
                    },
                )
                .unwrap();
            assert_eq!(placeholder_balance.balance.u128(), 867_u128);
            assert_eq!(investor1_currency_balance.balance.u128(), 430_u128);
            assert_eq!(investor2_currency_balance.balance.u128(), 866_u128);
            assert_eq!(issuer_balance.balance.u128(), 1337_u128);
            assert_eq!(investor1_bond_balance.balance.u128(), 180_u128);
            assert_eq!(investor2_bond_balance.balance.u128(), 378_u128);

            /* ================= Issuer sends coupons ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(ISSUER),
                    currency_address.clone(),
                    &Cw20ExecuteMsg::IncreaseAllowance {
                        spender: router_address.to_string(),
                        amount: Uint128::from(444_u128),
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ISSUER),
                    router_address.clone(),
                    &RouterExecuteMsg::SendCoupon {
                        bond_token: bond_token_address.to_string(),
                        coupons: vec![
                            Coupon {
                                investor: Addr::unchecked(INVESTOR_1).to_string(),
                                currency_amount: Uint128::from(123_u128),
                            },
                            Coupon {
                                investor: Addr::unchecked(INVESTOR_2).to_string(),
                                currency_amount: Uint128::from(321_u128),
                            },
                        ],
                    },
                    &[],
                )
                .unwrap();
            investor1_currency_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_1).to_string(),
                    },
                )
                .unwrap();
            investor2_currency_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_2).to_string(),
                    },
                )
                .unwrap();
            issuer_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(ISSUER).to_string(),
                    },
                )
                .unwrap();
            assert_eq!(investor1_currency_balance.balance.u128(), 553_u128);
            assert_eq!(investor2_currency_balance.balance.u128(), 1187_u128);
            assert_eq!(issuer_balance.balance.u128(), 893_u128);

            /* ================= Issuer returns the principals to investors ================= */
            let redemption_response: RedemptionAmountResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    bond_token_address.to_string(),
                    &BondTokenQueryMsg::AdditionalQueryMsg(
                        AdditionalQueryMsg::EstimateRedempmtionAmount {},
                    ),
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ISSUER),
                    currency_address.clone(),
                    &Cw20ExecuteMsg::IncreaseAllowance {
                        spender: router_address.to_string(),
                        amount: redemption_response.redemption_amount,
                        expires: None,
                    },
                    &[],
                )
                .unwrap();
            blockchain
                .execute_contract(
                    Addr::unchecked(ISSUER),
                    router_address.clone(),
                    &RouterExecuteMsg::Redeem {
                        bond_token: bond_token_address.to_string(),
                    },
                    &[],
                )
                .unwrap();
            investor1_currency_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_1).to_string(),
                    },
                )
                .unwrap();
            investor2_currency_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_2).to_string(),
                    },
                )
                .unwrap();
            issuer_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.clone(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(ISSUER).to_string(),
                    },
                )
                .unwrap();
            investor1_bond_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    bond_token_address.to_string(),
                    &BondTokenQueryMsg::Cw20QueryMsg(Cw20BaseQueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_1).to_string(),
                    }),
                )
                .unwrap();
            investor2_bond_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    bond_token_address.to_string(),
                    &BondTokenQueryMsg::Cw20QueryMsg(Cw20BaseQueryMsg::Balance {
                        address: Addr::unchecked(INVESTOR_2).to_string(),
                    }),
                )
                .unwrap();
            placeholder_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: placeholder_address.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(redemption_response.redemption_amount.u128(), 837_u128);
            assert_eq!(investor1_currency_balance.balance.u128(), 823_u128);
            assert_eq!(investor2_currency_balance.balance.u128(), 1754_u128);
            assert_eq!(issuer_balance.balance.u128(), 56_u128);
            assert_eq!(investor1_bond_balance.balance.u128(), 0_u128);
            assert_eq!(investor2_bond_balance.balance.u128(), 0_u128);
            assert_eq!(placeholder_balance.balance.u128(), 867_u128);

            /* ================= Admin withdraws system fee ================= */
            blockchain
                .execute_contract(
                    Addr::unchecked(ADMIN),
                    placeholder_address.clone(),
                    &PlaceholderExecuteMsg::WithdrawSystemFee {
                        recipient: Addr::unchecked(ADMIN).to_string(),
                    },
                    &[],
                )
                .unwrap();
            placeholder_balance = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: placeholder_address.to_string(),
                    },
                )
                .unwrap();
            let admin_balance: BalanceResponse = blockchain
                .wrap()
                .query_wasm_smart(
                    currency_address.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: Addr::unchecked(ADMIN).to_string(),
                    },
                )
                .unwrap();
            assert_eq!(placeholder_balance.balance.u128(), 0_u128);
            assert_eq!(admin_balance.balance.u128(), 867_u128);
        }
    }
}
