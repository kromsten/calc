use cosmwasm_std::{
    ensure, to_json_binary, Binary, Coin, CosmosMsg, Deps, QuerierWrapper, StdError, StdResult, Uint128, WasmMsg
};


use astrovault::{
    standard_pool::{
        handle_msg::ExecuteMsg as StandardExecute,
        query_msg::{
            QueryMsg as StandardQuery,
            PoolResponse as StandardPoolResponse,
            SimulationResponse
        }
    },
    ratio_pool::{
        handle_msg::ExecuteMsg as RatioExecute,
        query_msg::{
            QueryMsg as RatioQuery,
            PoolResponse as RatioPoolResponse,
        },
    },
    stable_pool::{
        handle_msg::ExecuteMsg as StableExecute, 
        query_msg::{
            QueryMsg as StableQuery, 
            PoolResponse as StablePoolResponse,
            StablePoolQuerySwapSimulation, 
        }
    },
    assets::{asset::{Asset, AssetInfo}, 
        ratio_pools::RatioPoolInfo, 
        pools::PoolInfo as StablePoolInfo, 
        pairs::PairInfo as StandardPoolInfo
    }, 
    ratio_pool_factory::query_msg::{SwapCalcResponse, QueryMsg as RatioFactoryQueryMsg},
    stable_pool_factory::query_msg::QueryMsg as StableFactoryQueryMsg,
    standard_pool_factory::query_msg::QueryMsg as StandardFactoryQueryMsg,
};

use crate::{state::config::get_router_config, types::{config::RouterConfig, pair::{Pair, PoolInfo, PoolType}}, ContractError};





pub fn ratio_pool_response(
    querier: &QuerierWrapper,
    contract_addr: &str,
) -> StdResult<RatioPoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &RatioQuery::Pool {}
    )
}


pub fn stable_pool_response(
    querier: &QuerierWrapper,
    contract_addr: &str,
) -> StdResult<StablePoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &StableQuery::Pool {}
    )
}

pub fn standard_pool_response(
    querier: &QuerierWrapper,
    contract_addr: &str,
) -> StdResult<StandardPoolResponse> {
    querier.query_wasm_smart(
        contract_addr, 
        &StandardQuery::Pool {}
    )
}


pub fn query_assets(
    querier: &QuerierWrapper,
    contract_addr: &str,
    pool_type: &PoolType,
) -> StdResult<Vec<Asset>> {
    let assets = match pool_type {
        PoolType::Stable => stable_pool_response(querier, contract_addr)?.assets,
        PoolType::Ratio => ratio_pool_response(querier, contract_addr)?.assets.into(),
        PoolType::Standard => standard_pool_response(querier, contract_addr)?.assets.into(),
    };
    Ok(assets)
}



pub fn query_ratio_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: [AssetInfo; 2]
) -> StdResult<RatioPoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &RatioFactoryQueryMsg::Pool { asset_infos }
    )
}

pub fn query_standard_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: [AssetInfo; 2]
) -> StdResult<StandardPoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &StandardFactoryQueryMsg::Pair { asset_infos }
    )
}

pub fn query_stable_pool_info(
    querier: &QuerierWrapper,
    contract_addr: &str,
    asset_infos: Vec<AssetInfo>
) -> StdResult<StablePoolInfo> {
    querier.query_wasm_smart(
        contract_addr, 
        &StableFactoryQueryMsg::Pool { asset_infos }
    )
}



pub fn pool_exist_in_registry(
    deps: Deps,
    pair: &Pair
) -> StdResult<bool> {

    let cfg : RouterConfig = get_router_config(deps.storage)?;

    let pool = pair.pool_info();
    let pool_type = pool.pool_type;

    let factory_address  = match pool_type {
        PoolType::Ratio => cfg.ratio_pool_factory,
        PoolType::Standard => cfg.standard_pool_factory,
        PoolType::Stable => cfg.stable_pool_factory,
    };

    ensure!(factory_address.is_some(), StdError::GenericErr {
        msg: format!("Factory address not set for pool type: {:?}", pool_type)
    });

    let factory_address = factory_address.as_ref().unwrap();

    let stable_assets = match pool_type {
        PoolType::Stable => query_assets(
                            &deps.querier, 
                            factory_address, 
                            &PoolType::Stable
                        )?
                        .iter()
                        .map(|a| a.info.clone())
                        .collect::<Vec<AssetInfo>>(),

        _ => vec![]
    };

    let pool_exists = match pool_type {
        PoolType::Ratio => query_ratio_pool_info(
            &deps.querier, 
            factory_address, 
            pair.assets()
        ).is_ok(),
        PoolType::Standard => query_standard_pool_info(
            &deps.querier, 
            factory_address, 
            pair.assets()
        ).is_ok(),
        PoolType::Stable => query_stable_pool_info(
            &deps.querier, 
            factory_address, 
            stable_assets
        ).is_ok(),
    };

    Ok(pool_exists)
}




impl PoolInfo {

    pub fn populate(&mut self, querier: &QuerierWrapper) -> Result<(), ContractError> {
        let assets = self.get_pool_assets(querier)?;

        let from_pos = assets.iter().position(|a| a.info == self.base_asset);
        ensure!(from_pos.is_some(), StdError::generic_err("Couldn't get asset info from the pool"));
        self.base_pool_index = Some(from_pos.unwrap() as u32);

        let to_pos = assets.iter().position(|a| a.info == self.quote_asset);
        ensure!(to_pos.is_some(), StdError::generic_err("Couldn't get asset info from the pool"));
        self.quote_pool_index = Some(to_pos.unwrap() as u32);
        Ok(())
    }


    pub fn validate(&self, deps: Deps) -> Result<(), ContractError> {
        deps.api.addr_validate(self.address.as_ref())?;
        ensure!(!self.base_asset.equal(&self.quote_asset), ContractError::SameAsset {});
        ensure!(self.base_asset.to_string().len() > 0, ContractError::EmptyAsset {});
        ensure!(self.quote_asset.to_string().len() > 0, ContractError::EmptyAsset {});

        let base_index = self.base_pool_index;
        let quote_index = self.quote_pool_index;

        match self.pool_type {
            PoolType::Stable => {
                ensure!(base_index.is_some() && quote_index.is_some(), 
                    StdError::generic_err("Stable pools must have both from and to asset indeces")
                );
                
            },
            PoolType::Ratio => {
                ensure!(base_index.is_some(), StdError::generic_err("Ratio pools must have from asset index"));
            },
            _ => {}
        }

        Ok(())
    }


    pub fn get_swap_simulate(
        &self,
        querier:               &QuerierWrapper,
        offer_asset:           Asset,
    ) -> StdResult<Uint128> {

        match self.pool_type {

            PoolType::Standard => {
                let res = querier.query_wasm_smart::<SimulationResponse>(
                    self.address.clone(), 
                    &StandardQuery::Simulation { offer_asset }
                )?;
                Ok(res.return_amount)
            }
            
            PoolType::Stable => {

                let (from_index, to_index) = self.from_to_indeces(&offer_asset.info);

                let msg = StableQuery::SwapSimulation { 
                    amount:                 offer_asset.amount, 
                    swap_from_asset_index:  from_index, 
                    swap_to_asset_index:    to_index.clone()
                };

                let to_index = to_index as usize;

                let res = querier.query_wasm_smart::<StablePoolQuerySwapSimulation>(
                    self.address.clone(), 
                    &msg
                )?;

                let swap_amount = res.swap_to_assets_amount.get(to_index).unwrap().clone();
                let mint_amount = res.mint_to_assets_amount.get(to_index).unwrap().clone();

                Ok(swap_amount.checked_add(mint_amount)?)

            },
            
            PoolType::Ratio => {

                let swap_from_asset_index = self.asset_index(&offer_asset.info) as u8;
            
                let msg = RatioQuery::SwapSimulation { 
                    amount: offer_asset.amount, 
                    swap_from_asset_index
                };
            
                let res = querier.query_wasm_smart::<SwapCalcResponse>(
                    self.address.clone(), 
                    &msg
                )?;
            
                Ok(res.to_amount_minus_fee)
            }
        }
    }


    pub fn asset_index(
        &self,
        offer_asset: &AssetInfo
    ) -> u32 {
        match self.base_asset == *offer_asset {
            true => self.base_pool_index.unwrap(),
            false => self.quote_pool_index.unwrap(),
        }
    }

    pub fn from_to_indeces(
        &self,
        offer_asset: &AssetInfo,
    ) -> (u32, u32) {
        match self.base_asset == *offer_asset {
            true => (self.base_pool_index.unwrap(), self.quote_pool_index.unwrap()),
            false => (self.quote_pool_index.unwrap(), self.base_pool_index.unwrap()),
        }
    }


    pub fn get_pool_assets(
        &self,
        querier: &QuerierWrapper,
    ) -> StdResult<Vec<Asset>> {
        query_assets(querier, &self.address, &self.pool_type)
    }



    pub fn pool_swap_binary_msg(
        &self,
        offer_asset:         Asset,
        expected_return:     Option<Uint128>,
    ) -> StdResult<Binary> {
        
        match self.pool_type {
            PoolType::Standard => {
                to_json_binary(&StandardExecute::Swap {
                    offer_asset,
                    expected_return,
                    belief_price: None,
                    max_spread: None,
                    to: None,
                })
            },
            PoolType::Stable => {
            
                to_json_binary(&StableExecute::Swap {
                    expected_return,
                    to: None,
                    swap_to_asset_index: self.asset_index(&offer_asset.info),
                })
            } 
            PoolType::Ratio => {
                to_json_binary(&RatioExecute::Swap {
                    expected_return,
                    to: None,
                })
            } 
        }
    }


    pub fn pool_swap_cosmos_msg(
        &self,
        offer_asset:         Asset,
        expected_return:     Option<Uint128>,
        funds:               Vec<Coin>,
    ) -> StdResult<CosmosMsg> {
        
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.address.clone(),
            msg: self.pool_swap_binary_msg(
                offer_asset, 
                expected_return, 
            )?,
            funds,
        }))
    }

}
