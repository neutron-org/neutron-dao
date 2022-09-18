# neutron-dao

This is very first version of neutron dao contract.  It has the simplest API, which implements 2 methods:

* query for the voting power of specific user;
* query all voting powers of all users.

Both of them currently return hardcoded values.

# Testing 

1. from `neutron` run: `make init`
2. run `./test_proposal.sh`
3. see that proposal has passed

# Contract key handlers

The DAO contract should only implement two handlers:

```rust
pub fn query_voting_power(deps: Deps, user_addr: Addr) -> StdResult<VotingPowerResponse> { ... }
```

```rust
pub fn query_voting_powers(deps: Deps) -> StdResult<Vec<VotingPowerResponse>> { ... }  
```

where ```VotingPowerResponse``` is defined as:

```rust
pub struct VotingPowerResponse {
    /// Address of the user
    pub user: String,
    /// The user's current voting power, i.e. the amount of NTRN tokens locked in voting contract
    pub voting_power: Uint128,
}
```