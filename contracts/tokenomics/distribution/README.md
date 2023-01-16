## Distribution Contract

The Distribution Contract is owned by DAO and is present in Genesis. The Distribution Schedule contract is responsible of the second step of token distribution where tokens sent to this contract are distributed between `share holders`, where `share holders` are a configurable set of addresses with number of shares.

The contract allows share holders to withdraw collected tokens. The contract can be configured only by DAO.

***There can be some minor amount of tokens that left after distribution (bc of math), these tokens are going to be left on contract.***

### Methods

- `transfer_ownership()` transfers the ownership [permissioned];
- `set_shares()` is called by the DAO to set the shares [permissioned];
- `fund()` is usually called by the Treasury contract to distribute funds [permissionless];
- `claim()` can be called by the shareholders  to receive the money [permissioned];
- `pause(duration)` pauses the contract for a given amount of time [permissioned, either Neutron DAO or the [Security subDAO](https://www.notion.so/Governance-Technical-Design-3ae3d16779ec4fe8b37df83ef2f052bc)];
- `unpause()` unpauses the contract [permissioned, only Neutron DAO].

### Queries

- `config()` returns current config;
- `pending()` returns a list of addresses with corresponding pending balance.
