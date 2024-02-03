use cosmwasm_std::{
    to_json_binary, Binary, Coin, CosmosMsg, QuerierWrapper, StdResult, Uint128
};


use astrovault::{
    assets::asset::{Asset, AssetInfo}, ratio_pool::{
        handle_msg::ExecuteMsg as RatioExecute,
        query_msg::{
            QueryMsg as RatioQuery,
            PoolResponse as RatioPoolResponse,
        },
    }, ratio_pool_factory::query_msg::SwapCalcResponse, router::state::{Hop as AstroHop, RatioHopInfo, StableHopInfo, StandardHopInfo}, stable_pool::{
        handle_msg::ExecuteMsg as StableExecute, 
        query_msg::{
            QueryMsg as StableQuery, 
            PoolResponse as StablePoolResponse,
            StablePoolQuerySwapSimulation, 
        }
    }, standard_pool::{
        handle_msg::ExecuteMsg as StandardExecute,
        query_msg::{
            QueryMsg as StandardQuery,
            PoolResponse as StandardPoolResponse,
            SimulationResponse
        }
    }
};


use crate::{
    types::{pool::{Pool, PoolType, PopulatedPool}, wrapper::ContractWrapper}, ContractError
};


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
) -> Result<Vec<Asset>, ContractError> {
    let assets = match pool_type {
        PoolType::Stable => stable_pool_response(querier, contract_addr)?.assets,
        PoolType::Ratio => ratio_pool_response(querier, contract_addr)?.assets.into(),
        PoolType::Standard => standard_pool_response(querier, contract_addr)?.assets.into(),
    };
    Ok(assets)
}




impl Pool {

    pub fn assets(&self) -> [AssetInfo; 2] {
        [self.base_asset.clone(), self.quote_asset.clone()]
    }

    pub fn denoms(&self) -> [String; 2] {
        [self.base_asset.to_string(), self.quote_asset.to_string()]
    }

    pub fn other_asset(&self, swap_asset: &AssetInfo) -> AssetInfo {
        if self.quote_asset.equal(swap_asset) {
            self.base_asset.clone()
        } else {
            self.quote_asset.clone()
        }
    }

}



impl PopulatedPool {


    pub fn assets(&self) -> [AssetInfo; 2] {
        [self.base_asset.clone(), self.quote_asset.clone()]
    }

    pub fn denoms(&self) -> [String; 2] {
        [self.base_asset.to_string(), self.quote_asset.to_string()]
    }

    pub fn other_asset(&self, offer_asset: &AssetInfo) -> AssetInfo {
        if self.quote_asset.equal(offer_asset) {
            self.base_asset.clone()
        } else {
            self.quote_asset.clone()
        }
    }

    pub fn other_denom(&self, offer_denom: &str) -> String {
        if self.quote_denom() == offer_denom {
            self.base_denom()
        } else {
            self.quote_denom()
        }
    }

    pub fn base_denom(&self) -> String {
        self.base_asset.to_string()
    }

    pub fn quote_denom(&self) -> String {
        self.quote_asset.to_string()
    }

    pub fn has_asset(&self, asset: &AssetInfo) -> bool {
        self.base_asset.equal(asset) || self.quote_asset.equal(asset)
    }

    pub fn has_denom(&self, denom: &str) -> bool {
        self.base_denom() == denom || self.quote_denom() == denom
    } 

    pub fn combined_denoms(&self, other: &PopulatedPool) -> [String; 3] {
        if other.has_denom(&self.quote_denom()) {
            [self.base_denom(), self.quote_denom(), other.other_denom(&self.quote_denom())]
        } else {
            [self.quote_denom(), self.base_denom(), other.other_denom(&self.base_denom())]
        }
    }

    pub fn common_asset(&self, other: &PopulatedPool) -> AssetInfo {
        if other.has_asset(&self.quote_asset) {
            self.quote_asset.clone()
        } else {
            self.base_asset.clone()
        }
    }

    pub fn uncommon_asset(&self, other: &PopulatedPool) -> AssetInfo {
        if other.has_asset(&self.quote_asset) {
            self.base_asset.clone()
        } else {
            self.quote_asset.clone()
        }
    }

    pub fn common_denom(&self, other: &PopulatedPool) -> String {
        self.common_asset(other).to_string()
    }

    pub fn asset_index(
        &self,
        offer_asset: &AssetInfo
    ) -> u32 {
        if self.quote_asset.equal(offer_asset) {
            self.base_index
        } else {
            self.quote_index
        }
    }

    pub fn from_to_indeces(
        &self,
        offer_asset: &AssetInfo,
    ) -> (u32, u32) {
        if self.quote_asset.equal(offer_asset) {
            (self.quote_index, self.base_index)
        } else {
            (self.base_index, self.quote_index)
        }
    }



    pub fn swap_simulation(
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


    

    pub fn swap_msg_binary(
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



    pub fn swap_msg_cosmos(
        &self,
        offer_asset:         Asset,
        expected_return:     Option<Uint128>,
        funds:               Vec<Coin>,
    ) -> StdResult<CosmosMsg> {

        let pair_contact = ContractWrapper(self.address.clone());
        
        let swap_msg = self.swap_msg_binary(
            offer_asset.clone(), 
            expected_return, 
        )?;

        if offer_asset.info.is_native_token() {
            pair_contact.execute(
                swap_msg, 
                funds
            )
        } else {
            pair_contact.execute_cw20(
                offer_asset.to_string(), 
                offer_asset.amount, 
                swap_msg
            )
        }
    }

    pub fn astro_hop(
        &self,
        querier:            &QuerierWrapper,
        offer_asset_info:   &AssetInfo,
    ) -> Result<AstroHop, ContractError> {

        let defaul_hop = AstroHop {
            mint_staking_derivative: None,
            ratio_hop_info: None,
            standard_hop_info: None,
            stable_hop_info: None,
        };

        let asset_infos = self.assets();

        let hop = match self.pool_type {
            PoolType::Ratio => AstroHop {
                    ratio_hop_info: Some(RatioHopInfo {
                        asset_infos,
                        from_asset_index: self.asset_index(offer_asset_info),
                    }),
                    ..defaul_hop
            },
            PoolType::Standard => AstroHop {
                    standard_hop_info: Some(StandardHopInfo {
                        offer_asset_info: offer_asset_info.clone(),
                        ask_asset_info: self.other_asset(offer_asset_info),
                    }),
                    ..defaul_hop
            },
            PoolType::Stable => {
                
                let (
                    from_asset_index, 
                    to_asset_index
                ) = self.from_to_indeces(offer_asset_info);

                // Stable astrop hop requires info about all assets in the pool
                // querying every time since complex to store in general pool info
                let asset_infos = query_assets(
                        querier, 
                        &self.address, 
                        &self.pool_type
                    )?
                    .iter()
                    .map(|a| a.info.clone()).collect::<Vec<AssetInfo>>();

                
                AstroHop {
                    stable_hop_info: Some(StableHopInfo {
                        from_asset_index,
                        to_asset_index,
                        asset_infos,
                    }),
                    ..defaul_hop
                }
            },
        };

        Ok(hop)
        
    }

}
