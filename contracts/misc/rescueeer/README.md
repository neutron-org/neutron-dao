# Rescueeer

<aside>
⚠️ The rescueeer contract is a **temporary** admin multisig which contains a method that allows anyone to transfer its power to the DAO once its time has expired. The rescueeer will only retain admin power during the first two weeks of Neutron’s life, after which anyone can remove its admin power.

</aside>

# The purpose

The only purpose of the Rescueeer is to fix possible misconfigurations between **the chain launch** and the **TGE start** without necessarily resorting to a chain halt. Ideally, the Rescueeer will self-destruct without ever having been used.

Neutron’s launch event and DAO infrastructure is a large, complex piece of code. We have created automated solutions to help configure them properly for launch, but any mistake left unattended could result in a misconfiguration. For the sake of safety and liveness, the Rescueeer provides a simple way to fix any remaining problem after launch, before users funds are at risk. 

# The details

It has 3 parameters:

1. Owner (the Multisig)
2. True Admin (the DAO)
3. End of Life (13 days after launch)

During its lifetime, the Rescueeer executes any message sent by the Owner.

```
┌────────────┐         ┌────────────┐
│ Multisig   │         │ Literally  │
│            │         │ anyone     │
└──────┬─────┘         └────────────┘
       │
       │Execute(Msg)
       │
┌──────▼─────┐         ┌────────────┐
│ Rescueeer  │  Msg    │ DAO        │
│            ├────────►│            │
└────────────┘         └────────────┘
```

**Once end of life is reached**, anyone can force Rescueeer to transfer its admin rights to the True Admin (e.g. the DAO). 

```
┌────────────┐         ┌────────────┐
│ Multisig   │         │ Literally  │
│            │    ┌────┤ anyone     │
└────────────┘    │    └────────────┘
                  │
        ┌─────────┘
        │ TransferAdmin(DAO)
        │
┌───────▼────┐         ┌────────────┐
│ Rescueeer  ├────────►│ DAO        │
│            │ Migrate │            │
└────────────┘ Admin(  └────────────┘
               DAO)
```

In an ideal scenario, all tests are successful and all contracts are properly configured. The Rescueeer is never used (it never executes any message from the multisig). Then, once the End of Life is reached, a third party triggers the return of admin powers to the Neutron Core DAO. 

If the Owner (the multisig) wants to explicitly drop the admin power before the End of Life, it can just “commit suicide”: e.g. update itself to brick the non-upgradable contract (Rescueeer will permissionlessly return admin rights to the DAO once the End of Life is reached anyway).

```
┌────────────┐         ┌────────────┐
│ Multisig   │         │ Literally  │
│            │         │ anyone     │
└─┬────────▲─┘         └────────────┘
  │        │
  └────────┘
   Suicide

┌────────────┐         ┌────────────┐
│ Rescueeer  │         │ DAO        │
│            │         │            │
└────────────┘         └────────────┘
```

### Recap

1. If everything is properly configured, the Owner does nothing in regard to DAO. It may commit suicide and Rescueeer will return admin rights without it once the End of Life is reached.
2. If there’s a reason to do something, the Owner is able to do that.

3 possible scenarios:

1. Nothing happens. End of Life comes and *someone* issues the transaction to transfer DAO admin back to DAO. Multisig never ever touched either DAO or Rescueeer.
    1. The same but Multisig decides that there’s no need of having the admin power. Then it just commits suicide before the End of Life.
2. If something happens, then Multisig is able to do stuff until the End of Life.

# Transferring the Rescueeer’s powers to the DAO

To transfer admin rights back to the DAO once the End of Life is reached, the following message has to be sent to the Rescueeer.

```
{
  transfer_admin: {
    address: "neutron1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrstdxvff"
  }
}
```

Where `neutron1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrstdxvff` is the DAO core module address.
