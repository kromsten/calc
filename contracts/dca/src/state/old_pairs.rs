use base::pair::OldPair;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const PAIRS: Map<Addr, OldPair> = Map::new("pairs_v1");
