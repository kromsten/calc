use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, Coin, CosmosMsg, StdResult, WasmMsg, Binary};
use astrovault::standard_pool::handle_msg::ExecuteMsg;

#[cw_serde]
pub struct PairContract(pub Addr);

impl PairContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        self.call_binary(msg, funds)
    }

    
    pub fn call_binary(&self, msg: Binary, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
    }
}
