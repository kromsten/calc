use cosmwasm_schema::write_api;

use astrovault_calc::msg::{ExecuteMsg, QueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
