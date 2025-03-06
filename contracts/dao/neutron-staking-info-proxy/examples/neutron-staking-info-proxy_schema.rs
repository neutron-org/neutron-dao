use cosmwasm_schema::write_api;
use neutron_staking_info_proxy::msg::{InstantiateMsg, MigrateMsg};
use neutron_staking_info_proxy_common::query::QueryMsg;
use neutron_staking_rewards_common::msg::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg
    }
}
