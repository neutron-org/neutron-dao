use crate::error::{ContractError, ContractResult};
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub vesting_lp_contract: Addr,
    pub atom_oracle_contract: Addr,
    pub usdc_oracle_contract: Addr,
    pub owner: Addr,
    pub manager: Option<Addr>,
}

impl Config {
    /// checks whether the config fields are valid.
    pub fn validate(&self) -> ContractResult<()> {
        if self.name.is_empty() {
            return Err(ContractError::NameIsEmpty {});
        }
        if self.description.is_empty() {
            return Err(ContractError::DescriptionIsEmpty {});
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::ContractError, types::Config};
    use cosmwasm_std::Addr;

    #[test]
    fn valid_config() {
        let cfg = Config {
            name: String::from("name"),
            description: String::from("description"),
            vesting_lp_contract: Addr::unchecked("vesting_lp_contract"),
            atom_oracle_contract: Addr::unchecked("atom_oracle_contract"),
            usdc_oracle_contract: Addr::unchecked("usdc_oracle_contract"),
            owner: Addr::unchecked("owner"),
            manager: None,
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn empty_name() {
        let cfg = Config {
            name: String::from(""),
            description: String::from("description"),
            vesting_lp_contract: Addr::unchecked("vesting_lp_contract"),
            atom_oracle_contract: Addr::unchecked("atom_oracle_contract"),
            usdc_oracle_contract: Addr::unchecked("usdc_oracle_contract"),
            owner: Addr::unchecked("owner"),
            manager: None,
        };
        assert_eq!(cfg.validate(), Err(ContractError::NameIsEmpty {}));
    }

    #[test]
    fn empty_description() {
        let cfg = Config {
            name: String::from("name"),
            description: String::from(""),
            vesting_lp_contract: Addr::unchecked("vesting_lp_contract"),
            atom_oracle_contract: Addr::unchecked("atom_oracle_contract"),
            usdc_oracle_contract: Addr::unchecked("usdc_oracle_contract"),
            owner: Addr::unchecked("owner"),
            manager: None,
        };
        assert_eq!(cfg.validate(), Err(ContractError::DescriptionIsEmpty {}));
    }
}
