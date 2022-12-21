BIN=neutrond

CORE_CONTRACT=./artifacts/cwd_subdao_core.wasm
PROPOSAL_SINGLE_CONTRACT=./artifacts/cwd_subdao_proposal_single.wasm
TIMELOCK_SINGLE_CONTRACT=./artifacts/cwd_subdao_timelock_single.wasm
CW4_VOTING_CONTRACT=./artifacts/cw4_voting.wasm
PRE_PROPOSE_SINGLE_CONTRACT=./artifacts/cwd_subdao_pre_propose_single.wasm

CHAIN_ID_1=test-1

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME_1=${NEUTRON_DIR}/data/test-1/

USERNAME_1=demowallet1
ADMIN_ADDR=$(${BIN} keys show ${USERNAME_1} -a --keyring-backend test --home ${HOME_1})

echo """
#############################################################################
#
# Uploading the subDAO contracts
#
#############################################################################
"""

## Upload the core contract
#RES=$(${BIN} tx wasm store ${CORE_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
#CORE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
#echo "CORE_CONTRACT_CODE_ID:" $CORE_CONTRACT_CODE_ID
#
## Upload the cw4 voting contract
#RES=$(${BIN} tx wasm store ${CW4_VOTING_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
#CW4_VOTING_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
#echo "CW4_VOTING_CONTRACT_CODE_ID:" $CW4_VOTING_CONTRACT_CODE_ID
#
## Upload the pre propose contract
#RES=$(${BIN} tx wasm store ${PRE_PROPOSE_SINGLE_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
#PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
#echo "PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID:" $PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID
#
## Upload the proposal contract
#RES=$(${BIN} tx wasm store ${PROPOSAL_SINGLE_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
#PROPOSAL_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
#echo "PROPOSAL_SINGLE_CONTRACT_CODE_ID:" $PROPOSAL_SINGLE_CONTRACT_CODE_ID

# Upload the timelock contract
RES=$(${BIN} tx wasm store ${TIMELOCK_SINGLE_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
TIMELOCK_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "TIMELOCK_SINGLE_CONTRACT_CODE_ID:" $TIMELOCK_SINGLE_CONTRACT_CODE_ID

echo """
#############################################################################
#
# Instantiating the timelock contract
#
#############################################################################
"""

INIT_TIMELOCK_SINGLE_CONTRACT='{
  "timelock_duration": 60,
  "owner": {
    "address": {
      "addr": "'"${ADMIN_ADDR}"'"
    }
  }
}'

RES=$(${BIN} tx wasm instantiate $TIMELOCK_SINGLE_CONTRACT_CODE_ID "$INIT_TIMELOCK_SINGLE_CONTRACT" --from ${USERNAME_1} --admin ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --label "init"  --keyring-backend test --gas-prices 0.0025stake --gas auto --gas-adjustment 1.4 --home ${HOME_1} --node tcp://127.0.0.1:16657)
TIMELOCK_SINGLE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[0].attributes[0].value')
echo "TIMELOCK_SINGLE_CONTRACT_ADDR:" $TIMELOCK_SINGLE_CONTRACT_ADDR

echo """
#############################################################################
#
# Instantiating the core subDAO contract
#
#############################################################################
"""

PRE_PROPOSE_INIT_MSG='{
  "deposit_info": {
    "denom": "untrn",
    "open_proposal_submission": false,
    "timelock_contract": "'"${TIMELOCK_SINGLE_CONTRACT_ADDR}"'"
  }
}'

PROPOSAL_SINGLE_INIT_MSG='{
  "threshold": 0.5,
  "max_voting_period": 600,
  "min_voting_period": 600,
  "allow_revoting": false,
  "close_proposal_on_execution_failure": true,
  "pre_propose_info": {
    "module_may_propose": {
      "info": {
        "code_id": "'"${PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID}"'",
        "label": "Neutron subDAO pre propose",
        "msg": "CnsKICAgImFsbG93X3Jldm90aW5nIjpmYWxzZSwKICAgInByZV9wcm9wb3NlX2luZm8iOnsKICAgICAgIk1vZHVsZU1heVByb3Bvc2UiOnsKICAgICAgICAgImluZm8iOnsKICAgICAgICAgICAgImNvZGVfaWQiOjUsCiAgICAgICAgICAgICJtc2ciOiAiZXdvZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FpWkdWd2IzTnBkRjlwYm1adklqcDdDaUFnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0prWlc1dmJTSTZld29nSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBaWRHOXJaVzRpT25zS0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSW1SbGJtOXRJanA3Q2lBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNKdVlYUnBkbVVpT2lKemRHRnJaU0lLSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdmUW9nSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNCOUNpQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lIMHNDaUFnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJbUZ0YjNWdWRDSTZJQ0l4TURBd0lpd0tJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWljbVZtZFc1a1gzQnZiR2xqZVNJNkltRnNkMkY1Y3lJS0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnZlN3S0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSW05d1pXNWZjSEp2Y0c5ellXeGZjM1ZpYldsemMybHZiaUk2Wm1Gc2MyVUtJQ0FnSUNBZ0lDQWdJQ0FnZlFvSyIsCiAgICAgICAgICAgICJsYWJlbCI6Im5ldXRyb24iCiAgICAgICAgIH0KICAgICAgfQogICB9LAogICAib25seV9tZW1iZXJzX2V4ZWN1dGUiOmZhbHNlLAogICAibWF4X3ZvdGluZ19wZXJpb2QiOnsKICAgICAgInRpbWUiOjYwNDgwMAogICB9LAogICAiY2xvc2VfcHJvcG9zYWxfb25fZXhlY3V0aW9uX2ZhaWx1cmUiOmZhbHNlLAogICAidGhyZXNob2xkIjp7CiAgICAgICJ0aHJlc2hvbGRfcXVvcnVtIjp7CiAgICAgICAgICJxdW9ydW0iOnsKICAgICAgICAgICAgInBlcmNlbnQiOiIwLjIwIgogICAgICAgICB9LAogICAgICAgICAidGhyZXNob2xkIjp7CiAgICAgICAgICAgICJtYWpvcml0eSI6ewogICAgICAgICAgICAgICAKICAgICAgICAgICAgfQogICAgICAgICB9CiAgICAgIH0KICAgfQp9"
      }
    }
  }
}'

DAO_INIT_MSG='{
  "description": "Neutron subDAO",
  "name": "Neutron subDAO",
  "initial_items": null,
  "proposal_modules_instantiate_info": [
    {
      "code_id": "'"${PROPOSAL_SINGLE_CONTRACT_CODE_ID}"'",
      "label": "DAO_Neutron_cw-proposal-single",
      "msg": "CnsKICAgImFsbG93X3Jldm90aW5nIjpmYWxzZSwKICAgInByZV9wcm9wb3NlX2luZm8iOnsKICAgICAgIk1vZHVsZU1heVByb3Bvc2UiOnsKICAgICAgICAgImluZm8iOnsKICAgICAgICAgICAgImNvZGVfaWQiOjUsCiAgICAgICAgICAgICJtc2ciOiAiZXdvZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FpWkdWd2IzTnBkRjlwYm1adklqcDdDaUFnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0prWlc1dmJTSTZld29nSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBaWRHOXJaVzRpT25zS0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSW1SbGJtOXRJanA3Q2lBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNKdVlYUnBkbVVpT2lKemRHRnJaU0lLSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdmUW9nSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNCOUNpQWdJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lIMHNDaUFnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWdJbUZ0YjNWdWRDSTZJQ0l4TURBd0lpd0tJQ0FnSUNBZ0lDQWdJQ0FnSUNBZ0lDQWljbVZtZFc1a1gzQnZiR2xqZVNJNkltRnNkMkY1Y3lJS0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnZlN3S0lDQWdJQ0FnSUNBZ0lDQWdJQ0FnSW05d1pXNWZjSEp2Y0c5ellXeGZjM1ZpYldsemMybHZiaUk2Wm1Gc2MyVUtJQ0FnSUNBZ0lDQWdJQ0FnZlFvSyIsCiAgICAgICAgICAgICJsYWJlbCI6Im5ldXRyb24iCiAgICAgICAgIH0KICAgICAgfQogICB9LAogICAib25seV9tZW1iZXJzX2V4ZWN1dGUiOmZhbHNlLAogICAibWF4X3ZvdGluZ19wZXJpb2QiOnsKICAgICAgInRpbWUiOjYwNDgwMAogICB9LAogICAiY2xvc2VfcHJvcG9zYWxfb25fZXhlY3V0aW9uX2ZhaWx1cmUiOmZhbHNlLAogICAidGhyZXNob2xkIjp7CiAgICAgICJ0aHJlc2hvbGRfcXVvcnVtIjp7CiAgICAgICAgICJxdW9ydW0iOnsKICAgICAgICAgICAgInBlcmNlbnQiOiIwLjIwIgogICAgICAgICB9LAogICAgICAgICAidGhyZXNob2xkIjp7CiAgICAgICAgICAgICJtYWpvcml0eSI6ewogICAgICAgICAgICAgICAKICAgICAgICAgICAgfQogICAgICAgICB9CiAgICAgIH0KICAgfQp9"
    }
  ],
  "vote_module_instantiate_info": {
    "code_id": "'"${CW4_VOTING_CONTRACT_CODE_ID}"'",
    "label": "DAO_Neutron_voting_registry",
    "msg": "ewogICAgICAibWFuYWdlciI6IG51bGwsCiAgICAgICJvd25lciI6IG51bGwsCiAgICAgICJ2b3RpbmdfdmF1bHQiOiAibmV1dHJvbjE0aGoydGF2cThmcGVzZHd4eGN1NDRydHkzaGg5MHZodWpydmNtc3RsNHpyM3R4bWZ2dzlzNWMyZXBxIgogICAgfQ=="
  }
}'

echo $DAO_INIT