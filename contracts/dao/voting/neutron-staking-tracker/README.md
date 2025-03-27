# **Neutron Staking Tracker**

The Neutron Staking Tracker is a contract that mirrors the state of bonded tokens from the Cosmos SDK Staking module.  
This contract allows you to query *historical* data about bonded tokens.  
To maintain consistency, the contract relies on callbacks to the `sudo` handler.

## **Why Use This Contract?**

This contract provides access to historical data about bonded tokens in the Cosmos SDK Staking module.  
This is particularly useful for other contracts, such as:

- **Staking vault contracts** (e.g., from DAO-DAO) — to calculate the voting power a user holds based on their bonded tokens.
- **Staking rewards contracts** — to accurately compute the rewards a user should accrue over time.
