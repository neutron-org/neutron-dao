use crate::permission::{Permission, PermissionType};
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

/// Defines a mapping from an address to a permission associated with the address.
pub const PERMISSIONS: Map<(Addr, PermissionType), Permission> = Map::new("permissions");
