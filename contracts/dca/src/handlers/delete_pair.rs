use crate::{
    error::ContractError,
    state::pairs::{delete_pair, find_pair},
};
use cosmwasm_std::{DepsMut, Response};

pub fn delete_pair_handler(deps: DepsMut, denoms: [String; 2]) -> Result<Response, ContractError> {
    let pair = find_pair(deps.storage, denoms)?;
    delete_pair(deps.storage, &pair);

    Ok(Response::new()
        .add_attribute("delete_pair", "true")
        .add_attribute("base_denom", pair.base_denom)
        .add_attribute("quote_denom", pair.quote_denom)
        .add_attribute("address", format!("{:#?}", pair.address)))
}

#[cfg(test)]
mod delete_pair_tests {
    use super::delete_pair_handler;
    use crate::{
        state::pairs::{find_pair, save_pair},
        types::pair::Pair,
    };
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn deletes_existing_pair() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        save_pair(deps.as_mut().storage, &pair).unwrap();

        let original_pair = find_pair(deps.as_ref().storage, pair.denoms()).unwrap();

        delete_pair_handler(deps.as_mut(), pair.denoms()).unwrap();

        assert_eq!(original_pair, pair);

        let err = find_pair(deps.as_ref().storage, pair.denoms()).unwrap_err();

        assert_eq!(err.to_string(), "dca::types::pair::Pair not found");
    }

    #[test]
    fn fails_to_delete_non_existent_pair() {
        let mut deps = mock_dependencies();

        let pair = Pair::default();

        let err = delete_pair_handler(deps.as_mut(), pair.denoms()).unwrap_err();

        assert_eq!(err.to_string(), "dca::types::pair::Pair not found");
    }
}
