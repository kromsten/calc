use test_context::test_context;

use crate::helpers::{
    chain::Chain,
    helper::{
        create_astro_standard_pool, init_derivative, instantiate_astro_standard_factory,
        instantiate_dca,
    },
};

#[test_context(Chain)]
#[test]
#[ignore]
fn test_instantiate_dca(chain: &mut Chain) {
    let user = chain.cfg.users[0].clone();
    instantiate_dca(chain, user.account.address, &user.key).unwrap();
}

#[test_context(Chain)]
#[test]
#[ignore]
fn test_instantiate_factories(chain: &mut Chain) {
    let user = chain.cfg.users[0].clone();
    instantiate_astro_standard_factory(chain, user.account.address.clone(), &user.key).unwrap();
    init_derivative(
        chain,
        user.account.address,
        "USDC".into(),
        "USDC".into(),
        18,
        &user.key,
    )
    .unwrap();
    create_astro_standard_pool(chain, &user.key).unwrap();
}
