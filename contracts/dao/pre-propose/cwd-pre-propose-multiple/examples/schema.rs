use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use cwd_pre_propose_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cwd_pre_propose_multiple::contract::ProposeMessage;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg<Empty>,
        execute: ExecuteMsg<ProposeMessage>,
    }
}
