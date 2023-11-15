use cosmwasm_schema::write_api;

use cwd_pre_propose_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use neutron_subdao_pre_propose_single::msg::QueryExt;
use neutron_subdao_pre_propose_single::types::ProposeMessage;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg<QueryExt>,
        execute: ExecuteMsg<ProposeMessage>,
    }
}
