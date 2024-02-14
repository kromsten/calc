use std::str::FromStr;

use super::chain::Chain;
use astrovault::assets::asset::AssetInfo;
use astrovault::ratio_pool_factory::handle_msg::RatioPoolSettings;
use astrovault::stable_pool_factory::handle_msg::{LockupConfig, StablePoolSettings};
use astrovault::staking_derivative::handle_msg::NetworkSettings;
use astrovault::staking_derivative::init_msg::{
    BulkSettings, CreateDxTokenSettings, DxTokenSettings,
};
use astrovault::standard_pool_factory::handle_msg::{BuybackburnSettings, PairSettings};
use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::{CosmwasmError, ProcessError};
use cosm_orc::orchestrator::{
    Address, ChainTxResponse, Coin as OrcCoin, ExecResponse, QueryResponse,
};
use cosm_orc::orchestrator::{InstantiateResponse, SigningKey};
use cosm_tome::chain::request::TxOptions;
use cosm_tome::modules::bank::model::SendRequest;
use cosmwasm_std::{from_json, Addr, Decimal, Decimal256, Timestamp, Uint128};

use cw20::Cw20Coin;
use dca::types::fee_collector::FeeCollector;
use serde::de::DeserializeOwned;
use serde::Serialize;

// contract names used by cosm-orc to register stored code ids / instantiated addresses:
pub const DCA: &str = "dca";
pub const CW20: &str = "cw20";

pub const ASTRO_STANDARD_PAIR: &str = "standard_pool";
pub const ASTRO_STABLE_PAIR: &str = "a_sta_pair";
pub const ASTRO_RATIO_PAIR: &str = "a_rat_pair";

pub const ASTRO_STANDARD_FACTORY: &str = "standard_factory";
pub const ASTRO_STABLE_FACTORY: &str = "a_sta_fact";
pub const ASTRO_RATIO_FACTORY: &str = "a_rat_fact";

pub const BULK: &str = "bulk_settings";
pub const DERIVATIVE: &str = "derivative";
pub const LP_STAKING: &str = "lp_staking";
pub const LOCKUPS: &str = "lockups";

pub const CREATION_FEE: u128 = 1_000_000_000;
pub const MINT_PRICE: u128 = 100_000_000;

pub fn instantiate_dca(
    chain: &mut Chain,
    admin: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let addr = Addr::unchecked(admin.clone());

    chain.orc.instantiate(
        DCA,
        "dca_instantiate",
        &dca::msg::InstantiateMsg {
            admin: addr.clone(),
            executors: vec![addr.clone()],
            fee_collectors: vec![FeeCollector {
                address: admin.clone(),
                allocation: Decimal::percent(100),
            }],
            default_swap_fee_percent: Decimal::percent(1),
            weighted_scale_swap_fee_percent: Decimal::percent(1),
            automation_fee_percent: Decimal::percent(1),
            default_page_limit: 4,
            paused: false,
            risk_weighted_average_escrow_level: Decimal::percent(1),
            twap_period: 0,
            default_slippage_tolerance: Decimal::percent(5),
            exchange_contract_address: Addr::unchecked("exchange"),
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}

pub fn instantiate_astro_standard_factory(
    chain: &mut Chain,
    admin: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let pair_code_id = chain.orc.contract_map.code_id(ASTRO_STANDARD_PAIR)?;
    let lp_staking_code_id = chain.orc.contract_map.code_id(LP_STAKING)?;
    let token_code_id = chain.orc.contract_map.code_id(CW20)?;

    chain.orc.instantiate(
        ASTRO_STANDARD_FACTORY,
        "astro_standard_factory_instantiate",
        &astrovault::standard_pool_factory::init_msg::InstantiateMsg {
            owner: Some(admin.clone()),
            pair_code_id,
            token_code_id,
            lp_staking_code_id,
            pair_settings: PairSettings {
                swap_fee: None,
                buybackburn_fee: Some(BuybackburnSettings {
                    fee: Decimal256::bps(20),
                    address: Addr::unchecked(admin.clone()),
                }),
            },
            is_create_pair_enabled: None,
            archway_handler_addr: None,
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}

pub fn instantiate_astro_stable_factory(
    chain: &mut Chain,
    admin: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let pool_code_id = chain.orc.contract_map.code_id(ASTRO_STABLE_PAIR)?;
    let lp_staking_code_id = chain.orc.contract_map.code_id(LP_STAKING)?;
    let token_code_id = chain.orc.contract_map.code_id(CW20)?;
    let lockups_code_id = chain.orc.contract_map.code_id(LOCKUPS)?;

    chain.orc.instantiate(
        ASTRO_STABLE_FACTORY,
        "astro_stable_factory_instantiate",
        &astrovault::stable_pool_factory::init_msg::InstantiateMsg {
            owner: Some(admin.clone()),
            token_code_id,
            lp_staking_code_id,
            archway_handler_addr: Some(admin.clone()),
            pool_code_id,
            lockups_code_id,
            pool_settings: StablePoolSettings {
                lockup: None,
                withdrawal_to_lockup: None,
                swap: None,
                collector_addr: Addr::unchecked(admin.clone()),
                max_deposit_unbalancing_threshold: None,
                xasset_mode_minter: None,
            },
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}

pub fn instantiate_astro_ratio_factory(
    chain: &mut Chain,
    admin: String,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let pool_code_id = chain.orc.contract_map.code_id(ASTRO_STABLE_PAIR)?;
    let lp_staking_code_id = chain.orc.contract_map.code_id(LP_STAKING)?;
    let token_code_id = chain.orc.contract_map.code_id(CW20)?;
    let lockups_code_id = chain.orc.contract_map.code_id(LOCKUPS)?;

    chain.orc.instantiate(
        ASTRO_RATIO_FACTORY,
        "astro_stable_factory_instantiate",
        &astrovault::ratio_pool_factory::init_msg::InstantiateMsg {
            owner: Some(admin.clone()),
            token_code_id,
            lp_staking_code_id,
            archway_handler_addr: Some(admin.clone()),
            pool_code_id,
            lockups_code_id,
            pool_settings: RatioPoolSettings {
                collector_addr: Addr::unchecked(admin.clone()),
                lockup: LockupConfig {
                    fee_decay_multiplier_nom: Uint128::one(),
                    fee_decay_multiplier_denom: Uint128::one(),
                    fee_decay_step_duration: 1,
                },
                max_deposit_unbalancing_threshold: Uint128::from(1000u128),
            },
            set_ratio_addr: admin.clone(),
            operator_addr: admin.clone(),
            ratio_expiration_time: 100,
            max_ratio_variation: Uint128::from(1000u128),
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}

pub fn init_derivative(
    chain: &mut Chain,
    admin: String,
    name: String,
    symbol: String,
    decimals: u8,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    let token_code_id = chain.orc.contract_map.code_id(CW20)?;
    let bulk_code_id = chain.orc.contract_map.code_id(BULK)?;
    let label = format!("{}-{}", name, symbol);

    chain.orc.instantiate(
        DERIVATIVE,
        "create_cw20_token",
        &astrovault::staking_derivative::init_msg::InstantiateMsg {
            owner: Some(admin.clone()),
            bulk_distributor_settings: BulkSettings {
                existing_address: None,
                code_id: Some(bulk_code_id),
                distribute_rewards_over: 86400,
            },
            dx_token_settings: DxTokenSettings {
                create_token: Some(CreateDxTokenSettings {
                    code_id: token_code_id,
                    name,
                    symbol,
                    label,
                    decimals,
                }),
                existing_address: None,
            },
            network_settings: NetworkSettings {
                native_asset_denom: "ustars".to_string(),
                unbonding_time: 1814400,
                window_time: 302400,
            },
            validators: vec![],
            external_source_operator: None,
            archway_handler_addr: None,
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}

pub fn init_token(
    chain: &mut Chain,
    admin: String,
    name: String,
    symbol: String,
    decimals: u8,
    key: &SigningKey,
) -> Result<InstantiateResponse, ProcessError> {
    chain.orc.instantiate(
        CW20,
        "create_cw20_token",
        &cw20_base::msg::InstantiateMsg {
            name,
            symbol,
            decimals,
            initial_balances: vec![
                Cw20Coin {
                    address: admin.clone(),
                    amount: Uint128::from(1_000_000_000000000000000000u128),
                },
                Cw20Coin {
                    address: chain.cfg.users[1].account.address.clone(),
                    amount: Uint128::from(1_000_000_000000000000000000u128),
                },
                Cw20Coin {
                    address: chain.cfg.users[2].account.address.clone(),
                    amount: Uint128::from(1_000_000_000000000000000000u128),
                },
            ],
            mint: None,
            marketing: None,
        },
        key,
        Some(admin.parse().unwrap()),
        vec![],
    )
}

pub fn create_astro_standard_pool(
    chain: &mut Chain,
    key: &SigningKey,
) -> Result<ExecResponse, ProcessError> {
    let token = chain.orc.contract_map.address(DERIVATIVE)?;

    chain.orc.execute(
        ASTRO_STANDARD_FACTORY,
        "create_astro_standard_pool",
        &astrovault::standard_pool_factory::handle_msg::ExecuteMsg::CreatePair {
            asset_infos: [
                AssetInfo::NativeToken {
                    denom: "ustars".to_string(),
                },
                AssetInfo::Token {
                    contract_addr: token,
                },
            ],
        },
        key,
        vec![],
    )
}

/* pub fn full_setup(
    chain: &mut Chain,
    admin: String,
    key: &SigningKey,
) -> Result<(), ProcessError> {


    Ok(())
}
 */

pub fn get_init_address(res: ChainTxResponse) -> String {
    res.find_event_tags("instantiate".to_string(), "_contract_address".to_string())[0]
        .value
        .clone()
}

pub fn wasm_query<S: Serialize>(
    chain: &mut Chain,
    address: &String,
    msg: &S,
) -> Result<QueryResponse, CosmwasmError> {
    let res = tokio_block(async {
        chain
            .orc
            .client
            .wasm_query(Address::from_str(&address)?, msg)
            .await
    });

    res
}

pub fn wasm_query_typed<R, S>(
    chain: &mut Chain,
    address: &String,
    msg: &S,
) -> Result<R, CosmwasmError>
where
    S: Serialize,
    R: DeserializeOwned,
{
    let res = tokio_block(async {
        chain
            .orc
            .client
            .wasm_query(Address::from_str(&address)?, msg)
            .await
    })?;

    let res: R = from_json(&res.res.data.unwrap()).unwrap();

    Ok(res)
}

// gen_users will create `num_users` random SigningKeys
// and then transfer `init_balance` of funds to each of them.
pub fn gen_users(
    chain: &mut Chain,
    num_users: u32,
    init_balance: u128,
    denom: Option<&String>,
) -> Vec<SigningKey> {
    let prefix = &chain.cfg.orc_cfg.chain_cfg.prefix;
    let base_denom = &chain.cfg.orc_cfg.chain_cfg.denom;
    let from_user = &chain.cfg.users[1];

    let mut users = vec![];
    for n in 0..num_users {
        users.push(SigningKey::random_mnemonic(n.to_string()));
    }

    let mut reqs = vec![];
    for user in &users {
        let mut amounts = vec![OrcCoin {
            amount: init_balance,
            denom: base_denom.parse().unwrap(),
        }];
        // add extra denom if specified
        if let Some(denom) = denom {
            amounts.push(OrcCoin {
                amount: init_balance,
                denom: denom.parse().unwrap(),
            });
        }
        reqs.push(SendRequest {
            from: from_user.account.address.parse().unwrap(),
            to: user.to_addr(prefix).unwrap(),
            amounts,
        });
    }

    tokio_block(
        chain
            .orc
            .client
            .bank_send_batch(reqs, &from_user.key, &TxOptions::default()),
    )
    .unwrap();

    users
}

pub fn latest_block_time(chain: &Chain) -> Timestamp {
    let now = tokio_block(chain.orc.client.tendermint_query_latest_block())
        .unwrap()
        .block
        .header
        .unwrap()
        .time
        .unwrap();

    Timestamp::from_seconds(now.seconds.try_into().unwrap())
}
