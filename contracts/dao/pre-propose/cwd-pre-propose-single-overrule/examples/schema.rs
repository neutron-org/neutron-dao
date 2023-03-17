use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::Addr;
use cwd_pre_propose_base::msg::{DepositInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use neutron_dao_pre_propose_overrule::msg::QueryExt;
use neutron_dao_pre_propose_overrule::types::ProposeMessage;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg<ProposeMessage>), &out_dir);
    export_schema(&schema_for!(QueryMsg<QueryExt>), &out_dir);
    export_schema(&schema_for!(DepositInfoResponse), &out_dir);

    export_schema_with_title(&schema_for!(Addr), &out_dir, "ProposalModuleResponse");
    export_schema_with_title(&schema_for!(Addr), &out_dir, "DaoResponse");
}
