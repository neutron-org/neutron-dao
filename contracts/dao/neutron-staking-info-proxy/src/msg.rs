use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub staking_vault: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Update contract configuration. Must be called by `owner`.
    UpdateConfig {
        owner: Option<String>,
        staking_vault: Option<String>,
    },
    ///
    UpdateStake { user: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the contract config (static parameters only).
    #[returns(ConfigResponse)]
    Config {},
}

/// Response for `QueryMsg::Config`
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub staking_vault: String,
}

#[cw_serde]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
