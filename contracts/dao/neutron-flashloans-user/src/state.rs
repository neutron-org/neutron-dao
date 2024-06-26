use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// Stores the execution mode of the neutron-flashloans-user contract. See msg.rs for more details.
pub const EXECUTION_MODE: Item<u64> = Item::new("neutron-flashloans-user-execution-mode");

/// Stores the address of the flashloans contract. It is necessary for the
/// MODE_REQUEST_ANOTHER_LOAN_RECURSIVELY execution mode.
pub const FLASHLOANS_CONTRACT: Item<Addr> =
    Item::new("neutron-flashloans-user-flashloans-contract");
