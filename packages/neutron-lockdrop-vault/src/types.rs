use crate::error::ContractError;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub lockdrop_contract: Addr,
    pub oracle_usdc_contract: Addr,
    pub oracle_atom_contract: Addr,
    pub owner: Addr,
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
            lockdrop_contract: Addr::unchecked("lockdrop_contract"),
            oracle_usdc_contract: Addr::unchecked("oracle_usdc_contract"),
            oracle_atom_contract: Addr::unchecked("oracle_atom_contract"),
            owner: Addr::unchecked("owner"),
        };
        assert_eq!(cfg_ok.validate(), Ok(()));

        let cfg_empty_name = Config {
            name: String::from(""),
            description: String::from("description"),
            lockdrop_contract: Addr::unchecked("lockdrop_contract"),
            oracle_usdc_contract: Addr::unchecked("oracle_usdc_contract"),
            oracle_atom_contract: Addr::unchecked("oracle_atom_contract"),
            owner: Addr::unchecked("owner"),
        };
        assert_eq!(
            cfg_empty_name.validate(),
            Err(ContractError::NameIsEmpty {})
        );

        let cfg_empty_description = Config {
            name: String::from("name"),
            description: String::from(""),
            lockdrop_contract: Addr::unchecked("lockdrop_contract"),
            oracle_usdc_contract: Addr::unchecked("oracle_usdc_contract"),
            oracle_atom_contract: Addr::unchecked("oracle_atom_contract"),
            owner: Addr::unchecked("owner"),
        };
        assert_eq!(
            cfg_empty_description.validate(),
            Err(ContractError::DescriptionIsEmpty {})
        );
    }
}
