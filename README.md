# neutron-dao

This is very first version of neutron dao contract.  It has the simplest API, which implements 2 methods: 
- query vote power of specific user
- query all vote powers of all users

Both of them currently return hardcoded values.


# Testing 

1. from `neutron` run: `make init`
2. run `./test_proposal.sh`
3. see that proposal has passed

# Contract key functions
 ```rust
pub fn query_voting_power(deps: Deps, user_addr: Addr) -> StdResult<VotingPowerResponse> {...}
```

```rust
pub fn query_voting_powers(deps: Deps) -> StdResult<Vec<VotingPowerResponse>>  {...}  
```
where ```VotingPowerResponse``` is

```rust
pub struct VotingPowerResponse {
    /// Address of the user
    pub user: String,
    /// The user's current voting power, i.e. the amount of NTRN tokens locked in voting contract
    pub voting_power: Uint128,
}
```
currently neutron-core uses only `query_voting_powers`, but `query_voting_power` seems to be useful in future
