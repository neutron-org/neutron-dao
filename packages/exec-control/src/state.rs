use crate::types::Config;
use cw_storage_plus::Item;

/// The height the contract is paused until. If it's None, the contract is not paused.
pub(crate) const PAUSED_UNTIL: Item<Option<u64>> = Item::new("exec_control_paused_until");

/// The current configuration of the module.
pub(crate) const CONFIG: Item<Config> = Item::new("exec_control_config");
