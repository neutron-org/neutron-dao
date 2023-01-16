## Treasury Contract

The Treasury contract is owned by DAO and is instantiated in Genesis. Treasury contract is responsible of the first step of tokens distribution where a fraction of vested of tokens goes to the Distribution Contract, and the the rest goes to the Reserve contract.

Treasury tokens are vested based on on-chain activity: `burnt_tokens * a_multiplier`. The multiplier is a linear function (fn(y) = x/10; denominator should be changed based on tests run coins consumption, right now it consumes ~13000000 on test run and it means that it will exhaust all treasury tokens very quickly @Spaydh). At the moment I found that this value should be at least 100_000_000_000 of the supply: while initially, one burnt tokens equals multiple NTRN tokens made liquid, the flow of new tokens into the Treasury progressively slows down until the tokens supply is exhausted and the tokenomy becomes deflationary.

Treasury contract can be configured only by DAO. The contract has the `min_period` parameter, and the `distribute` method can not be called more than once every `min_period`. The `distribute` call is permissionless and can be called by anybody.

### Methods

- `transfer_ownership()` transfers the ownership [permissioned];
- `distribute()` sends the money to the Distribution Contract [permissionless];
- `update_config()` updates the config [permissioned];
- `pause(duration)` pauses the contract for a given amount of time [permissioned, either Neutron DAO or the [Security subDAO](https://www.notion.so/Governance-Technical-Design-3ae3d16779ec4fe8b37df83ef2f052bc)];
- `unpause()` unpauses the contract [permissioned, only Neutron DAO].

### Queries

- `config()` returns current config;
- `stats()` returns statistics on tokens distribution, including the total amount of money sent to the Distribution Contract and the amount of tokens sent to Reserve Contract
