use cosmwasm_schema::write_api;

use exchange::msg::{ExecuteMsg, QueryMsg};
use fin::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
