use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const PROPOSALS: Map<(u64, Addr), u64> = Map::new("overrule_proposals");
