use crate::constants::{EXCHANGE_CONTRACT_ADDRESS, ONE, PAIR_CONTRACT_ADDRESS, SWAP_FEE_RATE, TEN};
use crate::helpers::price::{FinBookResponse, FinPoolResponse, FinSimulationResponse};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Binary, ContractResult, CustomQuery, Decimal,
    Decimal256, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, StdError, StdResult,
    SystemError, SystemResult, Timestamp, Uint256, WasmQuery,
};
use cw20::Denom;
use exchange::msg::{OrderStatus, QueryMsg as ExchangeQueryMsg};
use kujira::fin::{
    BookResponse, ConfigResponse, OrderResponse, PoolResponse, QueryMsg as FinQueryMsg,
    SimulationResponse,
};
use kujira::precision::Precision;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::str::FromStr;

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
                WasmQuery::Smart { msg, contract_addr } => match contract_addr.as_str() {
                    PAIR_CONTRACT_ADDRESS => match from_binary::<FinQueryMsg>(msg).unwrap() {
                        FinQueryMsg::Config {} => to_binary(&ConfigResponse {
                            owner: Addr::unchecked("pair-admin"),
                            denoms: [
                                Denom::Native(DENOM_UKUJI.to_string()),
                                Denom::Native(DENOM_UUSK.to_string()),
                            ],
                            price_precision: Precision::DecimalPlaces(3),
                            decimal_delta: 0,
                            is_bootstrapping: false,
                            fee_taker: Decimal256::percent(1),
                            fee_maker: Decimal256::percent(1),
                            fee_maker_negative: false,
                        })
                        .unwrap(),
                        FinQueryMsg::Book { .. } => to_binary(&BookResponse {
                            base: vec![PoolResponse {
                                quote_price: Decimal256::percent(100),
                                offer_denom: Denom::Native(DENOM_UKUJI.to_string()),
                                total_offer_amount: Uint256::from_uint128(TEN),
                            }],
                            quote: vec![PoolResponse {
                                quote_price: Decimal256::percent(100),
                                offer_denom: Denom::Native(DENOM_UUSK.to_string()),
                                total_offer_amount: Uint256::from_uint128(TEN),
                            }],
                        })
                        .unwrap(),
                        FinQueryMsg::Simulation { offer_asset } => to_binary(&SimulationResponse {
                            return_amount: offer_asset.amount.into(),
                            spread_amount: Uint256::from_uint128(
                                offer_asset.amount * Decimal::percent(5),
                            ),
                            commission_amount: Uint256::from_uint128(
                                offer_asset.amount * Decimal::from_str(SWAP_FEE_RATE).unwrap(),
                            ),
                        })
                        .unwrap(),
                        FinQueryMsg::Order { order_idx } => to_binary(&OrderResponse {
                            idx: order_idx,
                            owner: Addr::unchecked("pair-admin"),
                            quote_price: Decimal256::percent(200),
                            offer_denom: Denom::Native(DENOM_UUSK.to_string()),
                            offer_amount: Uint256::zero(),
                            filled_amount: ONE.into(),
                            created_at: Timestamp::default(),
                            original_offer_amount: ONE.into(),
                        })
                        .unwrap(),
                        _ => panic!("Unsupported fin query"),
                    },
                    EXCHANGE_CONTRACT_ADDRESS => {
                        match from_binary::<ExchangeQueryMsg>(msg).unwrap() {
                            ExchangeQueryMsg::GetOrderStatus { .. } => {
                                to_binary(&OrderStatus::Filled).unwrap()
                            }
                            _ => panic!("Unsupported exchange wrapper query"),
                        }
                    }
                    _ => panic!("Unsupported contract addr"),
                },
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
        self.mock_querier.update_wasm(|query| {
            SystemResult::Ok(ContractResult::Ok(match query {
                WasmQuery::Smart { msg, .. } => match from_binary::<FinQueryMsg>(msg).unwrap() {
                    FinQueryMsg::Config {} => to_binary(&ConfigResponse {
                        owner: Addr::unchecked("pair-admin"),
                        denoms: [
                            Denom::Native(DENOM_UKUJI.to_string()),
                            Denom::Native(DENOM_UUSK.to_string()),
                        ],
                        price_precision: Precision::DecimalPlaces(3),
                        decimal_delta: 0,
                        is_bootstrapping: false,
                        fee_taker: Decimal256::percent(1),
                        fee_maker: Decimal256::percent(1),
                        fee_maker_negative: false,
                    })
                    .unwrap(),
                    FinQueryMsg::Book { .. } => to_binary(&FinBookResponse {
                        base: vec![FinPoolResponse {
                            quote_price: *price,
                            total_offer_amount: TEN,
                        }],
                        quote: vec![FinPoolResponse {
                            quote_price: *price,
                            total_offer_amount: TEN,
                        }],
                    })
                    .unwrap(),
                    FinQueryMsg::Simulation { offer_asset } => to_binary(&FinSimulationResponse {
                        return_amount: offer_asset.amount * (Decimal::one() / *price),
                        spread_amount: offer_asset.amount
                            * (Decimal::one() / *price)
                            * Decimal::percent(5),
                    })
                    .unwrap(),
                    _ => panic!("Unsupported query"),
                },
                _ => panic!("Unsupported query"),
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
