use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub credits_contract_address: Addr,
    pub owner: Addr,
    pub airdrop_contract_address: Addr,
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
        Ok(())
    }
}

#[cw_serde]
pub struct TotalSupplyResponse {
    // Total supply of ucNTRNs for specified block height
    pub total_supply: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAO: Item<Addr> = Item::new("dao");
pub const DESCRIPTION: Item<String> = Item::new("description");

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
            credits_contract_address: Addr::unchecked("credits_contract"),
            airdrop_contract_address: Addr::unchecked("airdrop_contract"),
            owner: Addr::unchecked("owner"),
        };
        assert_eq!(cfg_ok.validate(), Ok(()));

        let cfg_empty_name = Config {
            name: String::from(""),
            description: String::from("description"),
            credits_contract_address: Addr::unchecked("credits_contract"),
            airdrop_contract_address: Addr::unchecked("airdrop_contract"),
            owner: Addr::unchecked("owner"),
        };
        assert_eq!(
            cfg_empty_name.validate(),
            Err(ContractError::NameIsEmpty {})
        );

        let cfg_empty_description = Config {
            name: String::from("name"),
            description: String::from(""),
            credits_contract_address: Addr::unchecked("credits_contract"),
            airdrop_contract_address: Addr::unchecked("airdrop_contract"),
            owner: Addr::unchecked("owner"),
        };
        assert_eq!(
            cfg_empty_description.validate(),
            Err(ContractError::DescriptionIsEmpty {})
        );
    }
}
