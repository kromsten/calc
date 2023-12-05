use cosm_orc::orchestrator::cosm_orc::tokio_block;
use cosm_orc::orchestrator::error::StoreError;
use cosm_orc::orchestrator::{CosmosgRPC, Key, SigningKey, AccessConfig, StoreCodeResponse, ChainTxResponse, TendermintRPC, Address};
use cosm_orc::{config::cfg::Config, orchestrator::cosm_orc::CosmOrc};
use cosm_tome::chain::request::TxOptions;
use cosm_tome::modules::cosmwasm::model::{StoreCodeRequest, StoreCodeProto};
use cosmwasm_std::Binary;
use once_cell::sync::OnceCell;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use test_context::TestContext;
use cosmrs::proto::Any;


static CONFIG: OnceCell<Cfg> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct Cfg {
    pub orc_cfg: Config,
    pub users: Vec<SigningAccount>,
    pub gas_report_dir: String,
}

#[derive(Clone, Debug)]
pub struct SigningAccount {
    pub account: Account,
    pub key: SigningKey,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub address: String,
    pub mnemonic: String,
}


#[derive(Clone, Debug)]
pub struct Chain {
    pub cfg: Cfg,
    pub orc: CosmOrc<TendermintRPC>,
}




impl TestContext for Chain {
    fn setup() -> Self {
        let cfg = CONFIG.get_or_init(global_setup).clone();
        let orc = CosmOrc::new_tendermint_rpc(cfg.orc_cfg.clone(), true).unwrap();
        Self { cfg, orc }
    }

    fn teardown(self) {
        let cfg = CONFIG.get().unwrap();
        save_gas_report(&self.orc, &cfg.gas_report_dir);
    }
}



pub fn store_static_contracts(
    orc: &mut CosmOrc<TendermintRPC>,
    cfg: &Config,
    wasm_dir: &str,
    key: &SigningKey,
    instantiate_perms: Option<AccessConfig>,
) -> Result<Vec<ChainTxResponse>, StoreError> {
    let mut responses = vec![];
    let wasm_path = Path::new(wasm_dir);

    for wasm in fs::read_dir(wasm_path).map_err(StoreError::wasmdir)? {
        let wasm_path = wasm?.path();

        if wasm_path.extension() == Some(OsStr::new("txt")) {

            let wasm_data = fs::read(&wasm_path).map_err(StoreError::wasmfile)?;
            
            let signer_addr = key.to_addr(&cfg.chain_cfg.prefix).unwrap();

            let req = StoreCodeProto {
                signer_addr,
                wasm_data: Binary::from_base64(String::from_utf8(wasm_data.clone()).unwrap().as_str()).unwrap().to_vec(),
                instantiate_perms: instantiate_perms.clone(),
            };

            let msg: Any = req.try_into()?;


            let res = tokio_block(async {

                let raw = orc.client.tx_sign(vec![msg], key, &TxOptions {
                    timeout_height: None,
                    fee: None,
                    memo: String::default(),
                }).await.unwrap();

                orc.client.tx_broadcast_block(&raw).await

            }).unwrap();


            let code_id = res
                .find_event_tags("store_code".to_string(), "code_id".to_string())
                .get(0)
                .unwrap()
                .value
                .parse::<u64>()
                .unwrap();



            let contract = wasm_path
                .file_stem()
                .ok_or(StoreError::InvalidWasmFileName)?
                .to_str()
                .ok_or(StoreError::InvalidWasmFileName)?;

       
            orc.contract_map
                .register_contract(contract.to_string(), code_id);


            responses.push(res);
        }
    }
    Ok(responses)
}



// global_setup() runs once before all of the tests:
// - loads cosm orc / test account config files
// - stores contracts on chain for all tests to reuse
fn global_setup() -> Cfg {
    env_logger::init();

    let config = env::var("CONFIG").expect("missing yaml CONFIG env var");
    let gas_report_dir = env::var("GAS_OUT_DIR").unwrap_or_else(|_| "gas_reports".to_string());

    let mut cfg = Config::from_yaml(&config).unwrap();
    
    
    let mut orc = CosmOrc::new_tendermint_rpc(cfg.clone(), true).unwrap();
    let accounts = test_accounts();

    // Poll for first block to make sure the node is up:
    orc.poll_for_n_blocks(1, Duration::from_millis(10_000), true)
        .unwrap();

    let skip_storage = env::var("SKIP_CONTRACT_STORE").unwrap_or_else(|_| "false".to_string());
    if !skip_storage.parse::<bool>().unwrap() {

        orc.store_contracts("../artifacts", &accounts[0].key, None)
            .unwrap();


        store_static_contracts(&mut orc, &cfg, "./static", &accounts[0].key, None).unwrap();

        save_gas_report(&orc, &gas_report_dir);

        // persist stored code_ids in CONFIG, so we can reuse for all tests
        cfg.contract_deploy_info = orc.contract_map.deploy_info().clone();
    }

    Cfg {
        orc_cfg: cfg,
        users: accounts,
        gas_report_dir,
    }
}

fn test_accounts() -> Vec<SigningAccount> {
    let bytes = fs::read("configs/test_accounts.json").unwrap();
    let accounts: Vec<Account> = serde_json::from_slice(&bytes).unwrap();

    let accs = accounts
        .into_iter()
        .map(|a| SigningAccount {
            account: a.clone(),
            key: SigningKey {
                name: a.name,
                key: Key::Mnemonic(a.mnemonic),
                //derivation_path: "".to_string(),
            },
        })
        .collect::<Vec<SigningAccount>>();

    accs

}

fn save_gas_report(orc: &CosmOrc<TendermintRPC>, gas_report_dir: &str) {
    let report = orc
        .gas_profiler_report()
        .expect("error fetching profile reports");

    let j: Value = serde_json::to_value(report).unwrap();

    let p = Path::new(gas_report_dir);
    if !p.exists() {
        fs::create_dir(p).unwrap();
    }

    let mut rng = rand::thread_rng();
    let file_name = format!("test-{}.json", rng.gen::<u32>());
    fs::write(p.join(file_name), j.to_string()).unwrap();
}
