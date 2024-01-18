use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Binary, Coin as CosmosCoin, ContractResult, CustomQuery,
    Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, StdError, StdResult, SystemError,
    SystemResult, Uint128, WasmQuery,
};
use osmosis_std::shim::Any;
use osmosis_std::types::cosmos::base::v1beta1::Coin;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::Pool as ConcentratedLiquidityPool;
use osmosis_std::types::osmosis::gamm::poolmodels::stableswap::v1beta1::Pool as StableSwapPool;
use osmosis_std::types::osmosis::gamm::v1beta1::{
    Pool as GammPool, PoolAsset, PoolParams, QueryCalcJoinPoolSharesResponse,
};
use osmosis_std::types::osmosis::poolmanager::v1beta1::{
    EstimateSwapExactAmountInResponse, PoolRequest, PoolResponse,
};
use osmosis_std::types::osmosis::twap::v1beta1::ArithmeticTwapResponse;
use prost::Message;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

use super::constants::{
    DENOM_STAKE, DENOM_UATOM, DENOM_UION, DENOM_UOSMO, DENOM_USDC, ONE_DECIMAL, SWAP_FEE_RATE, TEN,
};

pub type StargateHandler = dyn Fn(&str, &Binary) -> StdResult<Binary>;

pub struct CalcMockQuerier<C: DeserializeOwned = Empty> {
    default_stargate_handler: Box<StargateHandler>,
    stargate_handler: Box<StargateHandler>,
    mock_querier: MockQuerier<C>,
}

impl<C: DeserializeOwned> CalcMockQuerier<C> {
    pub fn new() -> Self {
        Self {
            default_stargate_handler: Box::new(|path, data| match path {
                "/osmosis.twap.v1beta1.Query/ArithmeticTwapToNow" => {
                    to_json_binary(&ArithmeticTwapResponse {
                        arithmetic_twap: ONE_DECIMAL.to_string(),
                    })
                }
                "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountIn" => {
                    to_json_binary(&EstimateSwapExactAmountInResponse {
                        token_out_amount: Uint128::new(1231232).to_string(),
                    })
                }
                "/osmosis.gamm.v1beta1.Query/CalcJoinPoolShares" => {
                    to_json_binary(&QueryCalcJoinPoolSharesResponse {
                        share_out_amount: TEN.to_string(),
                        tokens_out: vec![],
                    })
                }
                "/osmosis.poolmanager.v1beta1.Query/Pool" => {
                    let gamm_pools = vec![
                        GammPool {
                            id: 0,
                            pool_assets: vec![
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_UOSMO.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_UATOM.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                            ],
                            pool_params: Some(PoolParams {
                                swap_fee: "0.001".to_string(),
                                exit_fee: ".01".to_string(),
                                smooth_weight_change_params: None,
                            }),
                            ..GammPool::default()
                        },
                        GammPool {
                            id: 1,
                            pool_assets: vec![
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_UOSMO.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_UION.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                            ],
                            pool_params: Some(PoolParams {
                                swap_fee: SWAP_FEE_RATE.to_string(),
                                ..PoolParams::default()
                            }),
                            ..GammPool::default()
                        },
                        GammPool {
                            id: 2,
                            pool_assets: vec![
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_UION.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_USDC.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                            ],
                            pool_params: Some(PoolParams {
                                swap_fee: SWAP_FEE_RATE.to_string(),
                                ..PoolParams::default()
                            }),
                            ..GammPool::default()
                        },
                        GammPool {
                            id: 3,
                            pool_assets: vec![
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_STAKE.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_UOSMO.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                            ],
                            pool_params: Some(PoolParams {
                                swap_fee: SWAP_FEE_RATE.to_string(),
                                ..PoolParams::default()
                            }),
                            ..GammPool::default()
                        },
                        GammPool {
                            id: 4,
                            pool_assets: vec![
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_STAKE.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                                PoolAsset {
                                    token: Some(Coin {
                                        denom: DENOM_UION.to_string(),
                                        amount: TEN.to_string(),
                                    }),
                                    weight: TEN.to_string(),
                                },
                            ],
                            pool_params: Some(PoolParams {
                                swap_fee: SWAP_FEE_RATE.to_string(),
                                ..PoolParams::default()
                            }),
                            ..GammPool::default()
                        },
                    ];

                    let cl_pools = vec![
                        ConcentratedLiquidityPool {
                            id: 5,
                            token0: DENOM_UOSMO.to_string(),
                            token1: DENOM_UATOM.to_string(),
                            ..ConcentratedLiquidityPool::default()
                        },
                        ConcentratedLiquidityPool {
                            id: 6,
                            token0: DENOM_UOSMO.to_string(),
                            token1: DENOM_UION.to_string(),
                            ..ConcentratedLiquidityPool::default()
                        },
                        ConcentratedLiquidityPool {
                            id: 7,
                            token0: DENOM_UION.to_string(),
                            token1: DENOM_USDC.to_string(),
                            ..ConcentratedLiquidityPool::default()
                        },
                        ConcentratedLiquidityPool {
                            id: 8,
                            token0: DENOM_STAKE.to_string(),
                            token1: DENOM_UOSMO.to_string(),
                            ..ConcentratedLiquidityPool::default()
                        },
                        ConcentratedLiquidityPool {
                            id: 9,
                            token0: DENOM_STAKE.to_string(),
                            token1: DENOM_UION.to_string(),
                            ..ConcentratedLiquidityPool::default()
                        },
                    ];

                    let ss_pools = vec![
                        StableSwapPool {
                            id: 10,
                            pool_liquidity: vec![
                                Coin {
                                    denom: DENOM_UOSMO.to_string(),
                                    amount: TEN.to_string(),
                                },
                                Coin {
                                    denom: DENOM_UATOM.to_string(),
                                    amount: TEN.to_string(),
                                },
                            ],
                            ..StableSwapPool::default()
                        },
                        StableSwapPool {
                            id: 11,
                            pool_liquidity: vec![
                                Coin {
                                    denom: DENOM_UOSMO.to_string(),
                                    amount: TEN.to_string(),
                                },
                                Coin {
                                    denom: DENOM_UION.to_string(),
                                    amount: TEN.to_string(),
                                },
                            ],
                            ..StableSwapPool::default()
                        },
                        StableSwapPool {
                            id: 12,
                            pool_liquidity: vec![
                                Coin {
                                    denom: DENOM_UION.to_string(),
                                    amount: TEN.to_string(),
                                },
                                Coin {
                                    denom: DENOM_USDC.to_string(),
                                    amount: TEN.to_string(),
                                },
                            ],
                            ..StableSwapPool::default()
                        },
                        StableSwapPool {
                            id: 13,
                            pool_liquidity: vec![
                                Coin {
                                    denom: DENOM_STAKE.to_string(),
                                    amount: TEN.to_string(),
                                },
                                Coin {
                                    denom: DENOM_UOSMO.to_string(),
                                    amount: TEN.to_string(),
                                },
                            ],
                            ..StableSwapPool::default()
                        },
                        StableSwapPool {
                            id: 14,
                            pool_liquidity: vec![
                                Coin {
                                    denom: DENOM_STAKE.to_string(),
                                    amount: TEN.to_string(),
                                },
                                Coin {
                                    denom: DENOM_UION.to_string(),
                                    amount: TEN.to_string(),
                                },
                            ],
                            ..StableSwapPool::default()
                        },
                    ];

                    let pool_id = PoolRequest::decode(data.as_slice()).unwrap().pool_id;

                    to_json_binary(&PoolResponse {
                        pool: Some(Any {
                            type_url: match pool_id {
                                0..=4 => GammPool::TYPE_URL.to_string(),
                                5..=9 => ConcentratedLiquidityPool::TYPE_URL.to_string(),
                                10.. => StableSwapPool::TYPE_URL.to_string(),
                            },
                            value: match pool_id {
                                0..=4 => gamm_pools
                                    .iter()
                                    .find(|pool| pool.id == pool_id)
                                    .unwrap()
                                    .clone()
                                    .encode_to_vec(),
                                5..=9 => cl_pools
                                    .iter()
                                    .find(|pool| pool.id == pool_id)
                                    .unwrap()
                                    .clone()
                                    .encode_to_vec(),
                                10.. => ss_pools
                                    .iter()
                                    .find(|pool| pool.id == pool_id)
                                    .unwrap()
                                    .clone()
                                    .encode_to_vec(),
                            },
                        }),
                    })
                }
                _ => panic!("Unexpected path: {}", path),
            }),
            stargate_handler: Box::new(|_, __| {
                Err(StdError::generic_err(
                    "no custom stargate handler, should invoke the default handler",
                ))
            }),
            mock_querier: MockQuerier::<C>::new(&[]),
        }
    }
}

impl<C: CustomQuery + DeserializeOwned> Querier for CalcMockQuerier<C> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<C> = match from_json(bin_request) {
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

    pub fn update_balance(&mut self, address: Addr, balances: Vec<CosmosCoin>) {
        self.mock_querier.update_balance(address, balances);
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

impl<C: DeserializeOwned> Default for CalcMockQuerier<C> {
    fn default() -> Self {
        Self::new()
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
