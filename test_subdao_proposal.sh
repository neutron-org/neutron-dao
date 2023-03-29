BIN=neutrond
#
#CORE_CONTRACT=./artifacts/cwd_subdao_core.wasm
#PROPOSAL_SINGLE_CONTRACT=./artifacts/cwd_subdao_proposal_single.wasm
#TIMELOCK_SINGLE_CONTRACT=./artifacts/cwd_subdao_timelock_single.wasm
#CW4_VOTING_CONTRACT=./artifacts/cw4_voting.wasm  # Vanilla DAO DAO contract, compiled from original repo
#CW4_GROUP_CONTRACT=./artifacts/cw4_group.wasm # Vanilla cw-plus contract, compiled from original repo
#PRE_PROPOSE_SINGLE_CONTRACT=./artifacts/cwd_subdao_pre_propose_single.wasm

CHAIN_ID=test-1

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME=${NEUTRON_DIR}/data/test-1/

# NOTE: this username is used to execute all transactions. It is also used here as the
# timelock's owner.
ADMIN=demowallet1
ADMIN_ADDR=$(${BIN} keys show ${ADMIN} -a --keyring-backend test --home ${HOME})


CORE_CONTRACT_ADDR=neutron1k95lcrdzamyeu882dtuclrzqmv6ay0axfa3wng8jla0ty52tzn4qsvxcpk
PROPOSAL_SINGLE_CONTRACT_ADDR=neutron1qyl0j7a24amk8k8gcmvv07y2zjx7nkcwpk73js24euh64hkja6esg9jar3
PRE_PROPOSE_SINGLE_CONTRACT_ADDR=neutron1ell22k43hs2jtx8x50jz96agaqju5jwn87ued0mzcfglzlw6um0ssqx6x5
CW4_VOTE_CONTRACT_ADDR=neutron15v8jqq6aqhsuykdgdevx3qqcj9lp4h27ypsycds4cmv6er9qv0vs99alac
TIMELOCK_SINGLE_CONTRACT_ADDR=neutron1yev7tj6lm9lf6mc0y6sxvwscu8rq76zlr4t8s63c5wj4v8u847kssmkmny

echo """
#############################################################################
#
# PROPOSAL COMPLETE EXECUTION SCENARIO:
#   1. Publish proposal,
#   2. Vote \"yes\",
#   3. Execute proposal on the proposal contract level (sends to timelock),
#   4. Try to execute before timelock expires (nothing happens),
#   5. Try to execute after timelock expires (status changes to \"executed\").
#
#############################################################################
#"""

RES=$(${BIN} tx bank send ${ADMIN_ADDR} ${TIMELOCK_SINGLE_CONTRACT_ADDR}  5000untrn  -y --chain-id ${CHAIN_ID} --output json \
  --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test \
  --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Sent 5000untrn from ADMIN_ADDR to TIMELOCK_SINGLE_CONTRACT_ADDR, tx hash:" $(echo $RES | jq -r '.txhash')

PROPOSAL_MSG='{
  "propose":{
    "msg":{
      "propose":{
        "title":"TEST_TIMELOCK_PROPOSAL",
        "description":"A proposal to test the timelock functionality",
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
echo "> Execute the proposal (should send it to timelock), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:26657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "timelocked"  ]; then
  echo '> Proposal status (in timelock contract) is "timelocked", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

${BIN} tx wasm execute $TIMELOCK_SINGLE_CONTRACT_ADDR '{"execute_proposal": {"proposal_id": 1}}'  --from ${ADMIN_ADDR} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657 > /dev/null 2>&1
echo "> Tried to execute the proposal before timelock expires (suppressing output)"

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:26657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "timelocked"  ]; then
  echo '> Proposal status (in timelock contract) is still "timelocked", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

echo "> Waiting for 20 seconds for the timelock to expire..."
sleep 20

RES=$(${BIN} tx wasm execute $TIMELOCK_SINGLE_CONTRACT_ADDR '{"execute_proposal": {"proposal_id": 1}}' \
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Tried to execute the proposal *after* timelock expires, tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}'  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:26657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "executed"  ]; then
  echo '> Proposal status (in timelock contract) is "executed", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"executed\""
  exit 1
fi

echo """
#############################################################################
#
# PROPOSAL OVERRULE SCENARIO:
#   1. Publish proposal,
#   2. Vote \"yes\",
#   3. Execute proposal on the proposal contract level (sends to timelock),
#   4. Overrule proposal,
#   5. Try to execute after the proposal was overruled (returns an error).
#
#############################################################################
"""

RES=$(${BIN} tx bank send ${ADMIN_ADDR} ${TIMELOCK_SINGLE_CONTRACT_ADDR}  5000untrn  -y --chain-id ${CHAIN_ID} --output json \
  --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test \
  --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Sent 5000untrn from ADMIN_ADDR to TIMELOCK_SINGLE_CONTRACT_ADDR, tx hash:" $(echo $RES | jq -r '.txhash')

PROPOSAL_MSG='{
  "propose":{
    "msg":{
      "propose":{
        "title":"TEST_TIMELOCK_PROPOSAL",
        "description":"A proposal to test the timelock functionality",
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

RES=$(${BIN} tx wasm execute $PRE_PROPOSE_SINGLE_CONTRACT_ADDR "$PROPOSAL_MSG" --amount 10untrn --from ${ADMIN_ADDR} -y \
  --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Submitted proposal, tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"vote": {"proposal_id": 2, "vote":  "yes"}}' \
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Submitted a YES vote (1 / 1 members), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $PROPOSAL_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 2}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:26657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.proposal.status')
if [ $PROPOSAL_STATUS == "passed"  ]; then
  echo '> Proposal status (in proposal contract) is "passed", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"passed\""
  exit 1
fi

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"execute": {"proposal_id": 2}}'  --from ${ADMIN_ADDR} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "> Execute the proposal (should send it to timelock), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 2}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:26657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "timelocked"  ]; then
  echo '> Proposal status (in timelock contract) is "timelocked", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

# -------------------- OVERRULING --------------------

PROP='{
  "propose": {
    "msg": {
      "propose_overrule": {
          "timelock_contract": "'"${TIMELOCK_SINGLE_CONTRACT_ADDR}"'",
          "proposal_id": 2
      }
    }
  }
}'
# PROPOSAL 1 (to pass)
#propose proposal we're going to pass
RES=$(${BIN} tx wasm execute $PRE_PROPOSE_ADDRESS "$PROP" --amount 1000untrn --from ${USERNAME_1} -y \
  --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "propose proposal to be passed:"
echo $RES

#### vote YES from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\":  \"yes\"}}"  \
  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "vote YES from wallet1:"
echo $RES

#### vote YES from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\":  \"yes\"}}"  \
  --from ${USERNAME_2} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "vote YES from wallet2:"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"execute\": {\"proposal_id\": 1}}"  --from ${USERNAME_1} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657)
echo "execute proposal:"
echo $RES

# -------------------- TRYING TO EXECUTE --------------------

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 2}}'  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:26657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "overruled"  ]; then
  echo '> Proposal status (in timelock contract) is "overruled", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"overruled\""
  exit 1
fi

RES=$(${BIN} tx wasm execute $TIMELOCK_SINGLE_CONTRACT_ADDR '{"execute_proposal": {"proposal_id": 2}}' \
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025untrn \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:26657) > /dev/null 2>&1
echo "> Tried to execute the overruled proposal, should return an error"
grep -q 'Wrong proposal status (overruled)' <<< $RES && echo "> Received an error, all good" || echo "ERROR: proposal execution did not fail"
