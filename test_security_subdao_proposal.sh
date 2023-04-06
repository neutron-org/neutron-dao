BIN=neutrond

CHAIN_ID=test-1

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME=${NEUTRON_DIR}/data/test-1/

# NOTE: this username is used to execute all transactions.
ADMIN=demowallet1
ADMIN_ADDR=$(${BIN} keys show ${ADMIN} -a --keyring-backend test --home ${HOME})
SERNAME_3=rly1


CORE_CONTRACT_ADDR=neutron1qyygux5t4s3a3l25k8psxjydhtudu5lnt0tk0szm8q4s27xa980s27p0kg
PROPOSAL_SINGLE_CONTRACT_ADDR=neutron1ul4msjc3mmaxsscdgdtjds85rg50qrepvrczp0ldgma5mm9xv8yqrxv496
PRE_PROPOSE_SINGLE_CONTRACT_ADDR=neutron1hpgq5juh354nepq5wmwyddts3eex9t02rd4zrhqqv5nsrpht9r6s75j0kl

echo """
#############################################################################
#
# PROPOSAL COMPLETE EXECUTION SCENARIO:
#   1. Publish proposal,
#   2. Vote \"yes\",
#   3. Execute proposal.
#
#############################################################################
"""

PROPOSAL_MSG='{
  "propose":{
    "msg":{
      "propose":{
        "title":"TEST_SUBDAO_PROPOSAL",
        "description":"A proposal to test the security subdao functionality",
        "msgs":[
          {
            "bank":{
              "send":{
                "to_address":"'"${ADMIN_ADDR}"'",
                "amount":[
                  {
                    "denom":"untrn",
                    "amount":"10"
                  }
                ]
              }
            }
          }
        ]
      }
    }
  }
}
'

RES=$(${BIN} tx bank send ${ADMIN_ADDR} ${CORE_CONTRACT_ADDR}  5000untrn  -y --chain-id ${CHAIN_ID} --output json \
  --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test \
  --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Sent 5000untrn from ADMIN_ADDR to CORE_CONTRACT_ADDR, tx hash:" $(echo $RES | jq -r '.txhash')


RES=$(${BIN} tx wasm execute $PRE_PROPOSE_SINGLE_CONTRACT_ADDR "$PROPOSAL_MSG" --amount 10untrn --from ${ADMIN_ADDR} -y \
  --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Submitted proposal, tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"vote": {"proposal_id": 1, "vote":  "yes"}}' \
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Submitted a YES vote (1 / 1 members), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $PROPOSAL_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:26657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.proposal.status')
if [ $PROPOSAL_STATUS == "passed"  ]; then
  echo '> Proposal status (in proposal contract) is "passed", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"passed\""
  exit 1
fi

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"execute": {"proposal_id": 1}}'  --from ${ADMIN_ADDR} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Execute the proposal& tx hash:" $(echo $RES | jq -r '.txhash')
