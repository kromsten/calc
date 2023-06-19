use cosmwasm_schema::write_api;

use exchange::msg::{ExecuteMsg, QueryMsg};
use osmosis::msg::{InstantiateMsg, InternalMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        sudo: InternalMsg
    }
}
