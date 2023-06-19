use cosmwasm_schema::write_api;

use exchange::msg::{ExecuteMsg, QueryMsg};
use osmosis::msg::{InstantiateMsg, InternalMsg, MigrateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        migrate: MigrateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        sudo: InternalMsg
    }
}
