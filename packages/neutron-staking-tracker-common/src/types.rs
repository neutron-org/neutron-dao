use crate::error::ContractError;
use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_schema::serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};

/// Configuration settings for the smart contract.
///
/// This struct holds key details about the vault, including:
/// - `name`: The name of the vault.
/// - `description`: A short text description of the vault.
/// - `owner`: The address of the vault owner/admin.
/// - `denom`: The token denomination used for delegations and governance.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub name: String,
    pub description: String,
    pub owner: Addr,
    pub staking_proxy_info_contract_address: Option<Addr>,
}

impl Config {
    /// Validates whether the configuration parameters are correctly set.
    ///
    /// - Ensures that `name`, `description`, and `denom` are not empty.
    /// - If any field is invalid, returns an appropriate `ContractError`.
    ///
    /// Returns:
    /// - `Ok(())` if all fields are valid.
    /// - `Err(ContractError)` if validation fails.
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

/// Represents a validator in the staking system.
///
/// A validator is responsible for securing the network and participating in consensus.
/// Each validator has:
/// - `cons_address`: The **consensus address** (`valcons`), used for signing blocks.
/// - `oper_address`: The **operator address** (`valoper`), used for staking/delegation.
/// - `bonded`: Whether the validator is bonded (actively participating in consensus).
/// - `total_tokens`: Total staked tokens delegated to this validator.
/// - `total_shares`: Total delegation shares representing ownership over the staked tokens.
/// - `active`: Whether the validator is active in the network.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Validator {
    pub cons_address: Addr,
    pub oper_address: Addr,
    pub bonded: bool,
    /// The total amount of delegator shares for this validator.
    ///
    /// Stored as a `Uint128` to maintain compatibility with Cosmos SDK’s `sdk.Dec`, which is serialized
    /// as an integer without a decimal point (scaled by `10^18`).
    ///
    /// ### Why `Uint128`?
    /// - **Preserves Precision**: The Cosmos SDK already scales `sdk.Dec` values by `10^18`,
    ///   so `Uint128` naturally maintains precision.
    /// - **Avoids Unnecessary Transformations**: Using `Decimal` would require multiple conversions
    ///   between string representations and numeric types, adding complexity and inefficiency.
    /// - **Prevents Overflow Issues**: `Decimal` in CosmWasm has limits on large numbers
    ///   (e.g., `10M shares * 10^18` would overflow).
    ///
    /// ### Example:
    /// In Cosmos SDK:
    /// - `1.000000000000000000` (1 with 18 decimal places) is stored as `"1000000000000000000"`.
    /// - `10.500000000000000000` (10.5 with 18 decimal places) is stored as `"10500000000000000000"`.
    ///
    /// Since Cosmos SDK stores `sdk.Dec` values as large integers, using `Uint128` prevents
    /// unnecessary conversions.
    pub total_tokens: Uint128,
    pub total_shares: Uint128,
    pub active: bool,
}

impl Validator {
    pub fn remove_del_shares(&mut self, shares: Uint128) -> Result<(), ContractError> {
        let remaining_shares = self.total_shares.checked_sub(shares)?;

        if remaining_shares.is_zero() {
            self.total_tokens = Uint128::zero();
        } else {
            let undelegated_tokens = shares.multiply_ratio(self.total_tokens, self.total_shares);
            self.total_tokens = self.total_tokens.checked_sub(undelegated_tokens)?;
        }

        self.total_shares = remaining_shares;

        Ok(())
    }
}

/// Represents a delegation made by a user to a validator.
///
/// A delegation means that a **delegator** (user) has assigned their stake to a **validator**.
/// - `delegator_address`: The user's wallet address that owns the stake.
/// - `validator_address`: The operator address (`valoper`) of the validator receiving the delegation.
/// - `shares`: The amount of **delegation shares** received in exchange for staked tokens.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Delegation {
    pub delegator_address: Addr,
    pub validator_address: Addr,
    /// The amount of shares this delegator has in the validator.
    ///
    /// Stored as a `Uint128` for the same reasons as `Validator::total_shares`:
    /// - **Cosmos SDK Compatibility**: Delegator shares are serialized as large integers (scaled by `10^18`).
    /// - **Efficiency**: Avoids the need for complex conversions and floating-point arithmetic.
    /// - **Overflow Prevention**: Using `Decimal` would cause issues when working with large numbers
    ///   due to its internal scaling mechanism.
    ///
    /// ### Example:
    /// - `5.000000000000000000` shares in Cosmos SDK are stored as `"5000000000000000000"`.
    /// - `2.123456789000000000` shares are stored as `"2123456789000000000"`.
    ///
    /// Using `Uint128` directly eliminates unnecessary conversion steps while ensuring compatibility.
    pub shares: Uint128,
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::error::ContractError;
    use cosmwasm_std::Addr;

    /// Tests the validation logic for the `Config` struct.
    ///
    /// Ensures that empty fields are properly rejected with the correct `ContractError`.
    #[test]
    fn test_config_validate() {
        let cfg_ok = Config {
            name: String::from("name"),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            staking_proxy_info_contract_address: None,
        };
        assert_eq!(cfg_ok.validate(), Ok(()));

        let cfg_empty_name = Config {
            name: String::from(""),
            description: String::from("description"),
            owner: Addr::unchecked("owner"),
            staking_proxy_info_contract_address: None,
        };
        assert_eq!(
            cfg_empty_name.validate(),
            Err(ContractError::NameIsEmpty {})
        );

        let cfg_empty_description = Config {
            name: String::from("name"),
            description: String::from(""),
            owner: Addr::unchecked("owner"),
            staking_proxy_info_contract_address: None,
        };
        assert_eq!(
            cfg_empty_description.validate(),
            Err(ContractError::DescriptionIsEmpty {})
        );
    }
}
