use cosmwasm_schema::write_api;
use cwd_pre_propose_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use neutron_dao_pre_propose_overrule::msg::{ProposeMessage, QueryExt};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg<QueryExt>,
        execute: ExecuteMsg<ProposeMessage>,
    }
}
