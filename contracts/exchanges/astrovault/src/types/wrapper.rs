use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Addr, Binary, Coin, CosmosMsg, StdResult, Uint128, WasmMsg};

#[cw_serde]
pub struct ContractWrapper(pub Addr);

impl ContractWrapper {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn execute(&self, msg: Binary, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds,
        }
        .into())
    }

    pub fn execute_cw20(&self, 
        contract_addr: String,
        amount: Uint128,
        msg: Binary
    ) -> StdResult<CosmosMsg> {
        
        Ok(WasmMsg::Execute {
            contract_addr: contract_addr,
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Send {
                contract: self.addr().into(),
                amount,
                msg,
            })?,
            funds: vec![],
        }
        .into())
        
    }
}
