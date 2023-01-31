use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub description: String,
    pub lockdrop_contract: Addr,
    pub owner: Option<Addr>,
    pub manager: Option<Addr>,
}
