use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    // Use an Option to allow setting this value later in case of a cyclical dependency.
    pub staking_rewards: Option<String>,
    pub staking_denom: String,
    pub providers: Vec<String>,
}

#[cw_serde]
pub struct MigrateMsg {}
