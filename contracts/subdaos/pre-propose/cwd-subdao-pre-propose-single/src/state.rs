use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const TIMELOCK_MODULE: Item<Addr> = Item::new("timelock_contract_address");
