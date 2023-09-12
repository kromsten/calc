use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Binary, Coin, ContractResult, CustomQuery, Decimal,
    Decimal256, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, StdError, StdResult,
    SystemError, SystemResult, Uint128, Uint256, WasmQuery,
};
use exchange::msg::Order;
use exchange::msg::Pair;
use exchange::msg::QueryMsg as ExchangeQueryMsg;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

pub const USER: &str = "user";
pub const ADMIN: &str = "admin";
pub const FEE_COLLECTOR: &str = "fee_collector";
pub const VALIDATOR: &str = "validator";

pub const DENOM_UKUJI: &str = "ukuji";
pub const DENOM_UUSK: &str = "uusk";

pub type StargateHandler = dyn Fn(&str, &Binary) -> StdResult<Binary>;

pub struct CalcMockQuerier<C: DeserializeOwned = Empty> {
    default_stargate_handler: Box<StargateHandler>,
    stargate_handler: Box<StargateHandler>,
    mock_querier: MockQuerier<C>,
}

impl<C: DeserializeOwned> CalcMockQuerier<C> {
    pub fn new() -> Self {
        let mut querier = MockQuerier::<C>::new(&[]);

        querier.update_wasm(|query| {
            SystemResult::Ok(ContractResult::Ok(match query {
                WasmQuery::Smart { msg, .. } => {
                    match from_binary::<ExchangeQueryMsg>(msg).unwrap() {
                        ExchangeQueryMsg::GetPairs { .. } => {
                            to_binary(&vec![Pair::default()]).unwrap()
                        }
                        ExchangeQueryMsg::GetOrder { .. } => to_binary(&Order {
                            order_idx: Uint128::new(328472),
                            remaining_offer_amount: Coin {
                                amount: Uint256::zero().try_into().unwrap(),
                                denom: DENOM_UUSK.to_string(),
                            },
                        })
                        .unwrap(),
                        ExchangeQueryMsg::GetTwapToNow { .. } => {
                            to_binary(&Decimal256::percent(100)).unwrap()
                        }
                        ExchangeQueryMsg::GetExpectedReceiveAmount {
                            swap_amount,
                            target_denom,
                        } => to_binary(&Coin {
                            amount: swap_amount.amount * Decimal::percent(95),
                            denom: target_denom,
                        })
                        .unwrap(),
                        ExchangeQueryMsg::InternalQuery { .. } => {
                            unimplemented!("Internal query unsupported")
                        }
                    }
                }
                _ => panic!("Unsupported contract addr"),
            }))
        });

        Self {
            default_stargate_handler: Box::new(|_, __| {
                Err(StdError::generic_err("no default stargate handler"))
            }),
            stargate_handler: Box::new(|_, __| {
                Err(StdError::generic_err(
                    "no custom stargate handler, should invoke the default handler",
                ))
            }),
            mock_querier: querier,
        }
    }
}

impl<C: DeserializeOwned> Default for CalcMockQuerier<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: CustomQuery + DeserializeOwned> Querier for CalcMockQuerier<C> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<C> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl<C: CustomQuery + DeserializeOwned> CalcMockQuerier<C> {
    pub fn update_stargate<WH: 'static>(&mut self, stargate_handler: WH)
    where
        WH: Fn(&str, &Binary) -> StdResult<Binary>,
    {
        self.stargate_handler = Box::from(stargate_handler);
    }

    pub fn update_wasm<WH: 'static>(&mut self, wasm_handler: WH)
    where
        WH: Fn(&WasmQuery) -> QuerierResult,
    {
        self.mock_querier.update_wasm(wasm_handler);
    }

    pub fn update_fin_price(&mut self, price: &'static Decimal) {
        self.mock_querier.update_wasm(move |query| {
            SystemResult::Ok(ContractResult::Ok(match query {
                WasmQuery::Smart { msg, .. } => {
                    match from_binary::<ExchangeQueryMsg>(msg).unwrap() {
                        ExchangeQueryMsg::GetPairs { .. } => {
                            to_binary(&vec![Pair::default()]).unwrap()
                        }
                        ExchangeQueryMsg::GetOrder { .. } => to_binary(&Order {
                            order_idx: Uint128::new(328472),
                            remaining_offer_amount: Coin {
                                amount: Uint256::zero().try_into().unwrap(),
                                denom: DENOM_UUSK.to_string(),
                            },
                        })
                        .unwrap(),
                        ExchangeQueryMsg::GetTwapToNow { .. } => to_binary(&price).unwrap(),
                        ExchangeQueryMsg::GetExpectedReceiveAmount {
                            swap_amount,
                            target_denom,
                        } => to_binary(&Coin {
                            amount: swap_amount.amount
                                * (Decimal::one() / price)
                                * Decimal::percent(95),
                            denom: target_denom,
                        })
                        .unwrap(),
                        ExchangeQueryMsg::InternalQuery { .. } => {
                            unimplemented!("Internal query unsupported")
                        }
                    }
                }
                _ => panic!("Unsupported contract addr"),
            }))
        });
    }

    pub fn handle_query(&self, request: &QueryRequest<C>) -> QuerierResult {
        match &request {
            QueryRequest::Stargate { path, data } => SystemResult::Ok(ContractResult::Ok(
                (*self.stargate_handler)(path, data)
                    .unwrap_or_else(|_| (*self.default_stargate_handler)(path, data).unwrap()),
            )),
            _ => self.mock_querier.handle_query(request),
        }
    }
}

pub fn calc_mock_dependencies() -> OwnedDeps<MockStorage, MockApi, CalcMockQuerier, Empty> {
    OwnedDeps {
        storage: MockStorage::new(),
        api: MockApi::default(),
        querier: CalcMockQuerier::new(),
        custom_query_type: PhantomData,
    }
}
