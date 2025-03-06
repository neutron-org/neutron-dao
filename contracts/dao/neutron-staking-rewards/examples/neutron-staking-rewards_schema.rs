use cosmwasm_schema::write_api;
use neutron_staking_rewards::msg::{InstantiateMsg, MigrateMsg, QueryMsg};
use neutron_staking_rewards_common::msg::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
