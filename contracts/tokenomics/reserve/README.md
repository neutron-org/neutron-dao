## Reserve Contract

The Reserve Contract keeps the funds that have already been vested, but haven't been sent to the Distribution contract yet. Is is used by the neutron DAO for one-off payouts.

### Methods:

- `transfer_ownership()` transfers the ownership [permissioned];
- `payout(address, amount)` is used for One-off payments [permissioned];
- `pause(duration)` pauses the contract for a given amount of time [permissioned, either Neutron DAO or the [Security subDAO](https://www.notion.so/Governance-Technical-Design-3ae3d16779ec4fe8b37df83ef2f052bc)];
- `unpause()` unpauses the contract [permissioned, only Neutron DAO].

### **Queries:**

- `config()` returns current config;
