BIN=neutrond

CORE_CONTRACT=./artifacts/cwd_subdao_core.wasm
PROPOSAL_SINGLE_CONTRACT=./artifacts/cwd_subdao_proposal_single.wasm
TIMELOCK_SINGLE_CONTRACT=./artifacts/cwd_subdao_timelock_single.wasm
CW4_VOTING_CONTRACT=./artifacts/cw4_voting.wasm
CW4_GROUP_CONTRACT=./artifacts/cw4_group.wasm
PRE_PROPOSE_SINGLE_CONTRACT=./artifacts/cwd_subdao_pre_propose_single.wasm

CHAIN_ID=test-1

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME=${NEUTRON_DIR}/data/test-1/

# NOTE: this username is used to execute all transactions. It is also used here as the
# timelock's owner.
ADMIN=demowallet1
ADMIN_ADDR=$(${BIN} keys show ${ADMIN} -a --keyring-backend test --home ${HOME})

echo """
#############################################################################
#
# Uploading the subDAO contracts
#
#############################################################################
"""

# Upload the core contract (1 / 6)
RES=$(${BIN} tx wasm store ${CORE_CONTRACT} --from ${ADMIN} --gas 50000000 --chain-id ${CHAIN_ID} \
  --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
CORE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "CORE_CONTRACT_CODE_ID:" $CORE_CONTRACT_CODE_ID

# Upload the cw4 voting contract (2 / 6)
RES=$(${BIN} tx wasm store ${CW4_VOTING_CONTRACT} --from ${ADMIN} --gas 50000000 --chain-id ${CHAIN_ID} \
  --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
CW4_VOTE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "CW4_VOTE_CONTRACT_CODE_ID:" $CW4_VOTE_CONTRACT_CODE_ID

# Upload the cw4 group contract (3 / 6)
RES=$(${BIN} tx wasm store ${CW4_GROUP_CONTRACT} --from ${ADMIN} --gas 50000000 --chain-id ${CHAIN_ID} \
  --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
CW4_GROUP_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "CW4_GROUP_CONTRACT_CODE_ID:" $CW4_GROUP_CONTRACT_CODE_ID

# Upload the pre propose contract (4 / 6)
RES=$(${BIN} tx wasm store ${PRE_PROPOSE_SINGLE_CONTRACT} --from ${ADMIN} --gas 50000000 --chain-id ${CHAIN_ID} \
  --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID:" $PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID

# Upload the proposal contract (5 / 6)
RES=$(${BIN} tx wasm store ${PROPOSAL_SINGLE_CONTRACT} --from ${ADMIN} --gas 50000000 --chain-id ${CHAIN_ID} \
  --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
PROPOSAL_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "PROPOSAL_SINGLE_CONTRACT_CODE_ID:" $PROPOSAL_SINGLE_CONTRACT_CODE_ID

# Upload the timelock contract (6 / 6)
RES=$(${BIN} tx wasm store ${TIMELOCK_SINGLE_CONTRACT} --from ${ADMIN} --gas 50000000 --chain-id ${CHAIN_ID} \
  --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
TIMELOCK_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "TIMELOCK_SINGLE_CONTRACT_CODE_ID:" $TIMELOCK_SINGLE_CONTRACT_CODE_ID

echo """
#############################################################################
#
# Instantiating the core subDAO contract
#
#############################################################################
"""

# -------------------- PROPOSE { PRE-PROPOSE { TIMELOCK } } --------------------

TIMELOCK_SINGLE_CONTRACT_INIT_MSG='{
  "timelock_duration": 20,
  "owner": "'"${ADMIN_ADDR}"'"
}'
TIMELOCK_SINGLE_CONTRACT_INIT_MSG_BASE64=$(echo ${TIMELOCK_SINGLE_CONTRACT_INIT_MSG} | base64)

# PRE_PROPOSE_INIT_MSG will be put into the PROPOSAL_SINGLE_INIT_MSG
PRE_PROPOSE_INIT_MSG='{
  "deposit_info": {
    "denom": {
      "token": {
        "denom": {
          "native": "stake"
        }
      }
    },
    "amount": "10",
    "refund_policy": "always"
  },
  "open_proposal_submission": false,
  "timelock_module_instantiate_info": {
    "code_id": '"${TIMELOCK_SINGLE_CONTRACT_CODE_ID}"',
    "label": "Neutron subDAO timelock",
    "msg": "'"${TIMELOCK_SINGLE_CONTRACT_INIT_MSG_BASE64}"'"
  }
}'
PRE_PROPOSE_INIT_MSG_BASE64=$(echo ${PRE_PROPOSE_INIT_MSG} | base64)

PROPOSAL_SINGLE_INIT_MSG='{
  "threshold": {
    "absolute_count": {
      "threshold": "1"
    }
  },
  "max_voting_period": {
    "time": 60
  },
  "allow_revoting": false,
  "close_proposal_on_execution_failure": true,
  "pre_propose_info": {
    "ModuleMayPropose": {
      "info": {
        "code_id": '"${PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID}"',
        "label": "Neutron subDAO pre propose",
        "msg": "'"${PRE_PROPOSE_INIT_MSG_BASE64}"'"
      }
    }
  }
}'
PROPOSAL_SINGLE_INIT_MSG_BASE64=$(echo ${PROPOSAL_SINGLE_INIT_MSG} | base64)

# -------------------- VOTE MODULE --------------------

CW4_VOTE_INIT_MSG='{
  "cw4_group_code_id": '"${CW4_GROUP_CONTRACT_CODE_ID}"',
  "initial_members": [
    {
      "addr": "'"${ADMIN_ADDR}"'",
      "weight": 1
    }
  ]
}'
CW4_VOTE_INIT_MSG_BASE64=$(echo ${CW4_VOTE_INIT_MSG} | base64)

# -------------------- CORE MODULE --------------------

CORE_CONTRACT_INIT_MSG='{
  "name": "Neutron subDAO",
  "description": "Neutron subDAO",
  "initial_items": null,
  "vote_module_instantiate_info": {
    "code_id": '"${CW4_VOTE_CONTRACT_CODE_ID}"',
    "label": "Neutron subDAO vote module",
    "msg": "'"${CW4_VOTE_INIT_MSG_BASE64}"'"
  },
  "proposal_modules_instantiate_info": [
    {
      "code_id": '"${PROPOSAL_SINGLE_CONTRACT_CODE_ID}"',
      "label": "Neutron DAO proposal",
      "msg": "'"${PROPOSAL_SINGLE_INIT_MSG_BASE64}"'"
    }
  ]
}'

RES=$(${BIN} tx wasm instantiate $CORE_CONTRACT_CODE_ID "$CORE_CONTRACT_INIT_MSG" --from ${ADMIN} \
  --admin ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --label "init" \
  --keyring-backend test --gas-prices 0.0025stake --gas auto --gas-adjustment 1.4 --home ${HOME} --node tcp://127.0.0.1:16657)

CORE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[0].attributes[0].value')
echo "CORE_CONTRACT_ADDR:" $CORE_CONTRACT_ADDR

CW4_VOTE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[4].attributes[16].value')
echo "CW4_VOTE_CONTRACT_ADDR:" $CW4_VOTE_CONTRACT_ADDR

PROPOSAL_SINGLE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[4].attributes[24].value')
echo "PROPOSAL_SINGLE_CONTRACT_ADDR:" $PROPOSAL_SINGLE_CONTRACT_ADDR

TIMELOCK_SINGLE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[4].attributes[30].value')
echo "TIMELOCK_SINGLE_CONTRACT_ADDR:" $TIMELOCK_SINGLE_CONTRACT_ADDR

PRE_PROPOSE_SINGLE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[4].attributes[32].value')
echo "PRE_PROPOSE_SINGLE_CONTRACT_ADDR:" $PRE_PROPOSE_SINGLE_CONTRACT_ADDR

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
"""

RES=$(${BIN} tx bank send ${ADMIN_ADDR} ${TIMELOCK_SINGLE_CONTRACT_ADDR}  5000stake  -y --chain-id ${CHAIN_ID} --output json \
  --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test \
  --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Sent 5000stake from ADMIN_ADDR to TIMELOCK_SINGLE_CONTRACT_ADDR, tx hash:" $(echo $RES | jq -r '.txhash')

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
                    "denom":"stake",
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

RES=$(${BIN} tx wasm execute $PRE_PROPOSE_SINGLE_CONTRACT_ADDR "$PROPOSAL_MSG" --amount 10stake --from ${ADMIN_ADDR} -y \
  --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Submitted proposal, tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"vote": {"proposal_id": 1, "vote":  "yes"}}' \
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Submitted a YES vote (1 / 1 members), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $PROPOSAL_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:16657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.proposal.status')
if [ $PROPOSAL_STATUS == "passed"  ]; then
  echo '> Proposal status (in proposal contract) is "passed", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"execute": {"proposal_id": 1}}'  --from ${ADMIN_ADDR} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Execute the proposal (should send it to timelock), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:16657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "timelocked"  ]; then
  echo '> Proposal status (in timelock contract) is "timelocked", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

${BIN} tx wasm execute $TIMELOCK_SINGLE_CONTRACT_ADDR '{"execute_proposal": {"proposal_id": 1}}'  --from ${ADMIN_ADDR} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657 > /dev/null 2>&1
echo "> Tried to execute the proposal before timelock expires (suppressing output)"

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:16657)
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
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Tried to execute the proposal *after* timelock expires, tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 1}}'  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:16657)
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

RES=$(${BIN} tx bank send ${ADMIN_ADDR} ${TIMELOCK_SINGLE_CONTRACT_ADDR}  5000stake  -y --chain-id ${CHAIN_ID} --output json \
  --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test \
  --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Sent 5000stake from ADMIN_ADDR to TIMELOCK_SINGLE_CONTRACT_ADDR, tx hash:" $(echo $RES | jq -r '.txhash')

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
                    "denom":"stake",
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

RES=$(${BIN} tx wasm execute $PRE_PROPOSE_SINGLE_CONTRACT_ADDR "$PROPOSAL_MSG" --amount 10stake --from ${ADMIN_ADDR} -y \
  --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Submitted proposal, tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"vote": {"proposal_id": 2, "vote":  "yes"}}' \
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Submitted a YES vote (1 / 1 members), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $PROPOSAL_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 2}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:16657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.proposal.status')
if [ $PROPOSAL_STATUS == "passed"  ]; then
  echo '> Proposal status (in proposal contract) is "passed", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

RES=$(${BIN} tx wasm execute $PROPOSAL_SINGLE_CONTRACT_ADDR '{"execute": {"proposal_id": 2}}'  --from ${ADMIN_ADDR} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657)
echo "> Execute the proposal (should send it to timelock), tx hash:" $(echo $RES | jq -r '.txhash')

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 2}}' \
  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:16657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "timelocked"  ]; then
  echo '> Proposal status (in timelock contract) is "timelocked", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

${BIN} tx wasm execute $TIMELOCK_SINGLE_CONTRACT_ADDR '{"overrule_proposal": {"proposal_id": 2}}'  --from ${ADMIN_ADDR} \
  -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 \
  --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657 > /dev/null 2>&1
echo "> Overruled proposal"

RES=$(${BIN} q wasm contract-state smart $TIMELOCK_SINGLE_CONTRACT_ADDR '{"proposal": {"proposal_id": 2}}'  --chain-id ${CHAIN_ID} --output json  --home ${HOME} --node tcp://127.0.0.1:16657)
PROPOSAL_STATUS=$(echo $RES | jq -r '.data.status')
if [ $PROPOSAL_STATUS == "overruled"  ]; then
  echo '> Proposal status (in timelock contract) is "overruled", all good'
else
  echo "ERROR: Proposal status is \"${PROPOSAL_STATUS}\", should be \"timelocked\""
  exit 1
fi

RES=$(${BIN} tx wasm execute $TIMELOCK_SINGLE_CONTRACT_ADDR '{"execute_proposal": {"proposal_id": 2}}' \
  --from ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID} --output json --broadcast-mode=block --gas-prices 0.0025stake \
  --gas 1000000 --keyring-backend test --home ${HOME} --node tcp://127.0.0.1:16657) > /dev/null 2>&1
echo "> Tried to execute the overruled proposal, should return an error"
grep -q 'Wrong proposal status (overruled)' <<< $RES && echo "> Received an error, all good" || echo "ERROR: proposal execution did not fail"
