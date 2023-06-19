use crate::{msg::PairsResponse, state::config::get_config};
use cosmwasm_std::{Deps, StdResult};
use exchange::{msg::Pair, msg::QueryMsg};

pub fn get_pairs_handler(
    deps: Deps,
    limit: Option<u16>,
    start_after: Option<Pair>,
) -> StdResult<PairsResponse> {
    let config = get_config(deps.storage)?;
    Ok(PairsResponse {
        pairs: deps.querier.query_wasm_smart::<Vec<Pair>>(
            config.exchange_contract_address,
            &QueryMsg::GetPairs { limit, start_after },
        )?,
    })
}

#[cfg(test)]
mod get_pairs_tests {
    use crate::{
        contract::query,
        msg::{PairsResponse, QueryMsg},
        tests::{
            helpers::instantiate_contract,
            mocks::{calc_mock_dependencies, ADMIN},
        },
    };
    use cosmwasm_std::{
        from_binary,
        testing::{mock_dependencies, mock_env, mock_info},
        to_binary, ContractResult, SystemResult,
    };
    use exchange::msg::Pair;

    #[test]
    fn get_all_pairs_with_one_whitelisted_pair_should_succeed() {
        let mut deps = calc_mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info);

        let pair = Pair::default();

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Ok(
                to_binary::<Vec<Pair>>(&vec![Pair::default()]).unwrap(),
            ))
        });

        let response = from_binary::<PairsResponse>(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::GetPairs {
                    limit: None,
                    start_after: None,
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(response.pairs.len(), 1);
        assert_eq!(response.pairs[0], pair);
    }

    #[test]
    fn get_all_pairs_with_no_whitelisted_pairs_should_succeed() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN, &[]);

        instantiate_contract(deps.as_mut(), env.clone(), info);

        deps.querier.update_wasm(|_| {
            SystemResult::Ok(ContractResult::Ok(to_binary::<Vec<Pair>>(&vec![]).unwrap()))
        });

        let response = from_binary::<PairsResponse>(
            &query(
                deps.as_ref(),
                env,
                QueryMsg::GetPairs {
                    limit: None,
                    start_after: None,
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(response.pairs.len(), 0);
    }
}
