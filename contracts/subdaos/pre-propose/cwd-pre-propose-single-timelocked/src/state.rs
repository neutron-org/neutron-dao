use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const TIMELOCK_CONTRACT: Item<Addr> = Item::new("timelock_contract_address");
