use cosmwasm_std::{Deps, StdResult};
use exchange::msg::Pair;

use crate::state::pairs::get_exchange_pairs;

pub fn get_pairs_handler(
    deps: Deps,
    start_after: Option<Pair>,
    limit: Option<u16>,
) -> StdResult<Vec<Pair>> {
    Ok(get_exchange_pairs(
        deps.storage, 
        start_after.map(|pair| pair.denoms), 
        limit
    ))
}




#[cfg(test)]
mod get_pairs_handler {
    use crate::tests::common::{init_real_implicit, routed_pairs_real_implicit};

    use super::get_pairs_handler;
    

    #[test]
    fn get_real_work() {
        let data = init_real_implicit();

        let deps = data.deps.as_ref();
        let pairs = routed_pairs_real_implicit();

        let ex_pairs = get_pairs_handler(deps, None, None).unwrap();
        println!("Exp {:?}", ex_pairs);

        assert_eq!(pairs.len(), 2);    
    }
    

}