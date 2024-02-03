use crate::types::{pair::{Pair, PairType, PopulatedPair, PopulatedPairType, StoredPairType}, route::{HopInfo, PopulatedRoute, Route, RouteHop}};
use exchange::msg::Pair as ExchangePair;


impl Into<PairType> for PopulatedPairType {
    fn into(self) -> PairType {
        match self {
            PopulatedPairType::Direct { 
                address, 
                pool_type, 
                .. 
            } => PairType::Direct { address, pool_type },

            PopulatedPairType::Routed { 
                route 
            } => PairType::Routed { route: unpopulated_route(route) },
        }
    }
}



impl Into<Pair> for PopulatedPair {
    fn into(self) -> Pair {
        Pair {
            base_asset: self.base_asset,
            quote_asset: self.quote_asset,
            pair_type: self.pair_type.into(),
        }
    }
}

impl Into<ExchangePair> for PopulatedPair {
    fn into(self) -> ExchangePair {
        ExchangePair {
            denoms: self.denoms()
        }
    }
}

impl Into<StoredPairType> for PopulatedPairType {
    fn into(self) -> StoredPairType {
        match self {
            PopulatedPairType::Direct { .. } => StoredPairType::Direct,
            PopulatedPairType::Routed { .. } => StoredPairType::Routed,
        }
    }
}

impl Into<StoredPairType> for PopulatedPair {
    fn into(self) -> StoredPairType {
        self.pair_type.into()
    }
}

impl Into<StoredPairType> for &PopulatedPair {
    fn into(self) -> StoredPairType {
        self.pair_type.clone().into()
    }
}




fn unpopulated_route(route: PopulatedRoute) -> Route {
    // populated pool is at least 2 hops
    let mut hops : Vec<RouteHop> = Vec::with_capacity(route.len() - 1);    

    for (index, hop) in route.iter().enumerate().skip(1) {
        
        let prev = route.get(index - 1).unwrap();

        // if second to last hop, populate next
        let next = if index == route.len() - 2 {
            let next = route.last().unwrap();

            Some(HopInfo {
                address: next.address.clone(),
                pool_type: next.pool_type.clone(),
                asset_info: hop.uncommon_asset(next)
            })
        } else {
            None
        };

        hops.push(RouteHop {
            prev: HopInfo {
                address: prev.address.clone(),
                pool_type: prev.pool_type.clone(),
                asset_info: hop.uncommon_asset(prev)
            },
            next,
            denom: hop.common_denom(prev),
        });
    }

    hops
}