use cosmwasm_std::Empty;
use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::write_api;
use cwd_pre_propose_base::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cwd_pre_propose_single::contract::ProposeMessage;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg<Empty>,
        execute: ExecuteMsg<ProposeMessage>,
    }
}
