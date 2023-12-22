use cosmwasm_std::{Decimal, Env, QuerierWrapper, StdError, StdResult};
use osmosis_std::shim::Timestamp;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::Pool as ConcentratedLiquidityPool;
use osmosis_std::types::osmosis::cosmwasmpool::v1beta1::{
    CosmWasmPool, SpotPrice, SpotPriceQueryMsg, SpotPriceQueryMsgResponse,
};
use osmosis_std::types::osmosis::gamm::poolmodels::stableswap::v1beta1::Pool as StableSwapPool;
use osmosis_std::types::osmosis::gamm::v1beta1::Pool as GammPool;
use osmosis_std::types::osmosis::poolmanager::v1beta1::PoolmanagerQuerier;
use osmosis_std::types::osmosis::twap::v1beta1::TwapQuerier;
use prost::DecodeError;

pub fn get_arithmetic_twap_to_now(
    querier: &QuerierWrapper,
    env: Env,
    pool_id: u64,
    quote_denom: String,
    base_denom: String,
    period: u64,
) -> StdResult<Decimal> {
    PoolmanagerQuerier::new(querier)
        .pool(pool_id)?
        .pool
        .map_or(
            Err(StdError::generic_err("pool not found")),
            |pool| match pool.type_url.as_str() {
                GammPool::TYPE_URL
                | ConcentratedLiquidityPool::TYPE_URL
                | StableSwapPool::TYPE_URL => TwapQuerier::new(querier)
                    .arithmetic_twap_to_now(
                        pool_id,
                        base_denom,
                        quote_denom,
                        Some(Timestamp {
                            seconds: (env.block.time.seconds() - period) as i64,
                            nanos: 0,
                        }),
                    )
                    .map_err(|err| {
                        StdError::generic_err(format!(
                            "Failed to retrieve arithmetic twap for pool id {}. Error: {}",
                            pool_id, err
                        ))
                    })
                    .map(|r| r.arithmetic_twap),
                CosmWasmPool::TYPE_URL => pool
                    .try_into()
                    .map_err(|e: DecodeError| StdError::ParseErr {
                        target_type: CosmWasmPool::TYPE_URL.to_string(),
                        msg: e.to_string(),
                    })
                    .map(|pool: CosmWasmPool| {
                        querier
                            .query_wasm_smart::<SpotPriceQueryMsgResponse>(
                                pool.contract_address,
                                &SpotPriceQueryMsg {
                                    spot_price: Some(SpotPrice {
                                        quote_asset_denom: quote_denom,
                                        base_asset_denom: base_denom,
                                    }),
                                },
                            )
                            .map(|r| r.spot_price)
                            .map_err(|err| {
                                StdError::generic_err(format!(
                                    "Failed to retrieve spot price for pool id {}. Error: {}",
                                    pool_id, err
                                ))
                            })
                    })
                    .unwrap(),
                _ => Err(StdError::generic_err(format!(
                    "pool type {} not supported",
                    pool.type_url
                ))),
            },
        )?
        .parse::<Decimal>()
}
