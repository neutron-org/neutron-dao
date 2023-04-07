use cosmwasm_std::Decimal;

pub struct DistributionParams {
    pub distribution_rate: Option<Decimal>,
    pub min_period: Option<u64>,
    pub vesting_denominator: Option<u128>,
}
