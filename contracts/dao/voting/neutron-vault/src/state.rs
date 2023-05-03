use crate::error::ContractError;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, SnapshotItem, SnapshotMap, Strategy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub owner: Addr,
    pub denom: String,
}

impl Config {
    /// checks whether the config fields are valid.
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.name.is_empty() {
            return Err(ContractError::NameIsEmpty {});
        };
        if self.description.is_empty() {
            return Err(ContractError::DescriptionIsEmpty {});
        };
        if self.denom.is_empty() {
            return Err(ContractError::DenomIsEmpty {});
        };
        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
pub const BONDED_BALANCES: SnapshotMap<&Addr, Uint128> = SnapshotMap::new(
    "bonded_balances",
    "bonded_balance__checkpoints",
    "bonded_balance__changelog",
    Strategy::EveryBlock,
);

pub const BONDED_TOTAL: SnapshotItem<Uint128> = SnapshotItem::new(
    "total_bonded",
    "total_bonded__checkpoints",
    "total_bonded__changelog",
    Strategy::EveryBlock,
);

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::error::ContractError;
    use cosmwasm_std::Addr;

    #[test]
    fn test_config_validate() {
        let cfg_ok = Config {
            name: String::from("name"),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            denom: String::from("denom"),
        };
        assert_eq!(cfg_ok.validate(), Ok(()));

        let cfg_empty_name = Config {
            name: String::from(""),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            denom: String::from("denom"),
        };
        assert_eq!(
            cfg_empty_name.validate(),
            Err(ContractError::NameIsEmpty {})
        );

        let cfg_empty_description = Config {
            name: String::from("name"),
            description: String::from(""),
            owner: Addr::unchecked("owner"),
            denom: String::from("denom"),
        };
        assert_eq!(
            cfg_empty_description.validate(),
            Err(ContractError::DescriptionIsEmpty {})
        );

        let cfg_empty_denom = Config {
            name: String::from("name"),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            denom: String::from(""),
        };
        assert_eq!(
            cfg_empty_denom.validate(),
            Err(ContractError::DenomIsEmpty {})
        );
    }
}
