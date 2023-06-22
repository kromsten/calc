use cosmwasm_schema::write_api;

use exchange::msg::{ExecuteMsg, QueryMsg};
use osmosis::msg::{InstantiateMsg, InternalExternalMsg, MigrateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        migrate: MigrateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        sudo: InternalExternalMsg
    }
}
