BIN=neutrond

CORE_CONTRACT=./artifacts/cwd_subdao_core.wasm
PROPOSAL_SINGLE_CONTRACT=./artifacts/cwd_subdao_proposal_single.wasm
TIMELOCK_SINGLE_CONTRACT=./artifacts/cwd_subdao_timelock_single.wasm
CW4_VOTING_CONTRACT=./artifacts/cw4_voting.wasm
CW4_GROUP_CONTRACT=./artifacts/cw4_group.wasm
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

# Upload the core contract (1 / 6)
RES=$(${BIN} tx wasm store ${CORE_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
CORE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "CORE_CONTRACT_CODE_ID:" $CORE_CONTRACT_CODE_ID

# Upload the cw4 voting contract (2 / 6)
RES=$(${BIN} tx wasm store ${CW4_VOTING_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
CW4_VOTE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "CW4_VOTE_CONTRACT_CODE_ID:" $CW4_VOTE_CONTRACT_CODE_ID

# Upload the cw4 group contract (3 / 6)
RES=$(${BIN} tx wasm store ${CW4_GROUP_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
CW4_GROUP_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "CW4_GROUP_CONTRACT_CODE_ID:" $CW4_GROUP_CONTRACT_CODE_ID

# Upload the pre propose contract (4 / 6)
RES=$(${BIN} tx wasm store ${PRE_PROPOSE_SINGLE_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID:" $PRE_PROPOSE_SINGLE_CONTRACT_CODE_ID

# Upload the proposal contract (5 / 6)
RES=$(${BIN} tx wasm store ${PROPOSAL_SINGLE_CONTRACT} --from ${USERNAME_1} --gas 50000000 --chain-id ${CHAIN_ID_1} --broadcast-mode=block --gas-prices 0.0025stake -y --output json  --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
PROPOSAL_SINGLE_CONTRACT_CODE_ID=$(echo $RES | jq -r '.logs[0].events[1].attributes[0].value')
echo "PROPOSAL_SINGLE_CONTRACT_CODE_ID:" $PROPOSAL_SINGLE_CONTRACT_CODE_ID

# Upload the timelock contract (6 / 6)
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

TIMELOCK_SINGLE_CONTRACT_INIT_MSG='{
  "timelock_duration": 60,
  "owner": {
    "address": {
      "addr": "'"${ADMIN_ADDR}"'"
    }
  }
}'

RES=$(${BIN} tx wasm instantiate $TIMELOCK_SINGLE_CONTRACT_CODE_ID "$TIMELOCK_SINGLE_CONTRACT_INIT_MSG" --from ${USERNAME_1} --admin ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --label "init"  --keyring-backend test --gas-prices 0.0025stake --gas auto --gas-adjustment 1.4 --home ${HOME_1} --node tcp://127.0.0.1:16657)
TIMELOCK_SINGLE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[0].attributes[0].value')
echo "TIMELOCK_SINGLE_CONTRACT_ADDR:" $TIMELOCK_SINGLE_CONTRACT_ADDR

echo """
#############################################################################
#
# Instantiating the core subDAO contract
#
#############################################################################
"""

# -------------------- PROPOSE { PRE-PROPOSE } --------------------

# PRE_PROPOSE_INIT_MSG will be put into the PROPOSAL_SINGLE_INIT_MSG
PRE_PROPOSE_INIT_MSG='{
  "deposit_info": {
    "denom": {
      "token": {
        "denom": {
          "native": "untrn"
        }
      }
    },
    "amount": "10",
    "refund_policy": "always"
  },
  "open_proposal_submission": false,
  "timelock_contract": "'"${TIMELOCK_SINGLE_CONTRACT_ADDR}"'"
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
      "weight": 10
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
      "label": "DAO_Neutron_cw-proposal-single",
      "msg": "'"${PROPOSAL_SINGLE_INIT_MSG_BASE64}"'"
    }
  ]
}'

RES=$(${BIN} tx wasm instantiate $CORE_CONTRACT_CODE_ID "$CORE_CONTRACT_INIT_MSG" --from ${USERNAME_1} --admin ${ADMIN_ADDR} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --label "init"  --keyring-backend test --gas-prices 0.0025stake --gas auto --gas-adjustment 1.4 --home ${HOME_1} --node tcp://127.0.0.1:16657)


echo ${RES}
# prints {"height":"2579","txhash":"59F8409A770D887A52F7807438AFFF4A39D40453B3B61C95F768289C939BC2A9","codespace":"","code":0,"data":"0A700A282F636F736D7761736D2E7761736D2E76312E4D7367496E7374616E7469617465436F6E747261637412440A426E657574726F6E31333070776A67356572667237376C7265636C6537347A347377363268367578633076666D3038656179636C76777A7A3265736D73353966393734","raw_log":"[{\"events\":[{\"type\":\"execute\",\"attributes\":[{\"key\":\"_contract_address\",\"value\":\"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2\"},{\"key\":\"_contract_address\",\"value\":\"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2\"}]},{\"type\":\"instantiate\",\"attributes\":[{\"key\":\"_contract_address\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"code_id\",\"value\":\"156\"},{\"key\":\"_contract_address\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"code_id\",\"value\":\"157\"},{\"key\":\"_contract_address\",\"value\":\"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2\"},{\"key\":\"code_id\",\"value\":\"158\"},{\"key\":\"_contract_address\",\"value\":\"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a\"},{\"key\":\"code_id\",\"value\":\"160\"},{\"key\":\"_contract_address\",\"value\":\"neutron1xm8emwrnvuc59f2mjk9gd6rpzsr9fm9jvtpvmwr3cz4lcq20qjcqul5su9\"},{\"key\":\"code_id\",\"value\":\"159\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/cosmwasm.wasm.v1.MsgInstantiateContract\"},{\"key\":\"module\",\"value\":\"wasm\"},{\"key\":\"sender\",\"value\":\"neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2\"}]},{\"type\":\"reply\",\"attributes\":[{\"key\":\"_contract_address\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"_contract_address\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"_contract_address\",\"value\":\"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a\"},{\"key\":\"_contract_address\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"}]},{\"type\":\"wasm\",\"attributes\":[{\"key\":\"_contract_address\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"action\",\"value\":\"instantiate\"},{\"key\":\"sender\",\"value\":\"neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2\"},{\"key\":\"_contract_address\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"action\",\"value\":\"instantiate\"},{\"key\":\"_contract_address\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"group_contract_address\",\"value\":\"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2\"},{\"key\":\"_contract_address\",\"value\":\"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2\"},{\"key\":\"action\",\"value\":\"add_hook\"},{\"key\":\"hook\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"sender\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"_contract_address\",\"value\":\"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2\"},{\"key\":\"action\",\"value\":\"update_admin\"},{\"key\":\"admin\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"sender\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"_contract_address\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"voting_regsitry_module\",\"value\":\"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa\"},{\"key\":\"_contract_address\",\"value\":\"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a\"},{\"key\":\"action\",\"value\":\"instantiate\"},{\"key\":\"dao\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"_contract_address\",\"value\":\"neutron1xm8emwrnvuc59f2mjk9gd6rpzsr9fm9jvtpvmwr3cz4lcq20qjcqul5su9\"},{\"key\":\"dao\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"deposit_info\",\"value\":\"Some(CheckedDepositInfo { denom: Native(\\"untrn\\"), amount: Uint128(10), refund_policy: Always })\"},{\"key\":\"method\",\"value\":\"instantiate\"},{\"key\":\"open_proposal_submission\",\"value\":\"false\"},{\"key\":\"proposal_module\",\"value\":\"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a\"},{\"key\":\"_contract_address\",\"value\":\"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a\"},{\"key\":\"update_pre_propose_module\",\"value\":\"neutron1xm8emwrnvuc59f2mjk9gd6rpzsr9fm9jvtpvmwr3cz4lcq20qjcqul5su9\"},{\"key\":\"_contract_address\",\"value\":\"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974\"},{\"key\":\"prop_module\",\"value\":\"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a\"}]}]}]","logs":[{"msg_index":0,"log":"","events":[{"type":"execute","attributes":[{"key":"_contract_address","value":"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2"},{"key":"_contract_address","value":"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2"}]},{"type":"instantiate","attributes":[{"key":"_contract_address","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"code_id","value":"156"},{"key":"_contract_address","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"code_id","value":"157"},{"key":"_contract_address","value":"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2"},{"key":"code_id","value":"158"},{"key":"_contract_address","value":"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a"},{"key":"code_id","value":"160"},{"key":"_contract_address","value":"neutron1xm8emwrnvuc59f2mjk9gd6rpzsr9fm9jvtpvmwr3cz4lcq20qjcqul5su9"},{"key":"code_id","value":"159"}]},{"type":"message","attributes":[{"key":"action","value":"/cosmwasm.wasm.v1.MsgInstantiateContract"},{"key":"module","value":"wasm"},{"key":"sender","value":"neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"}]},{"type":"reply","attributes":[{"key":"_contract_address","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"_contract_address","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"_contract_address","value":"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a"},{"key":"_contract_address","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"}]},{"type":"wasm","attributes":[{"key":"_contract_address","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"action","value":"instantiate"},{"key":"sender","value":"neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"},{"key":"_contract_address","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"action","value":"instantiate"},{"key":"_contract_address","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"group_contract_address","value":"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2"},{"key":"_contract_address","value":"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2"},{"key":"action","value":"add_hook"},{"key":"hook","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"sender","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"_contract_address","value":"neutron1ru04cr0cq8we8yjgaq286l84h93cmu0e6fhpwcwd4np9lsjq3eksf5tqa2"},{"key":"action","value":"update_admin"},{"key":"admin","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"sender","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"_contract_address","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"voting_regsitry_module","value":"neutron1dndprqjve6yyum4m5dptkem8z679f82jhsqppk9sdw9wf6trh4hqcwhqaa"},{"key":"_contract_address","value":"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a"},{"key":"action","value":"instantiate"},{"key":"dao","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"_contract_address","value":"neutron1xm8emwrnvuc59f2mjk9gd6rpzsr9fm9jvtpvmwr3cz4lcq20qjcqul5su9"},{"key":"dao","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"deposit_info","value":"Some(CheckedDepositInfo { denom: Native(\"untrn\"), amount: Uint128(10), refund_policy: Always })"},{"key":"method","value":"instantiate"},{"key":"open_proposal_submission","value":"false"},{"key":"proposal_module","value":"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a"},{"key":"_contract_address","value":"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a"},{"key":"update_pre_propose_module","value":"neutron1xm8emwrnvuc59f2mjk9gd6rpzsr9fm9jvtpvmwr3cz4lcq20qjcqul5su9"},{"key":"_contract_address","value":"neutron130pwjg5erfr77lrecle74z4sw62h6uxc0vfm08eayclvwzz2esms59f974"},{"key":"prop_module","value":"neutron10qdttl5qnqfsuvm852zrfysqkuydn0hwz6xe5472tv5590v95zhssyxa8a"}]}]}],"info":"","gas_wanted":"1389910","gas_used":"1007059","tx":null,"timestamp":"","events":[{"type":"coin_spent","attributes":[{"key":"c3BlbmRlcg==","value":"bmV1dHJvbjFtOWwzNTh4dW5oaHdkczA1Njh6YTQ5bXpodnV4eDl1eDh4YWZ4Mg==","index":true},{"key":"YW1vdW50","value":"MzQ3NXN0YWtl","index":true}]},{"type":"coin_received","attributes":[{"key":"cmVjZWl2ZXI=","value":"bmV1dHJvbjE3eHBmdmFrbTJhbWc5NjJ5bHM2Zjg0ejNrZWxsOGM1bDV4MnozNg==","index":true},{"key":"YW1vdW50","value":"MzQ3NXN0YWtl","index":true}]},{"type":"transfer","attributes":[{"key":"cmVjaXBpZW50","value":"bmV1dHJvbjE3eHBmdmFrbTJhbWc5NjJ5bHM2Zjg0ejNrZWxsOGM1bDV4MnozNg==","index":true},{"key":"c2VuZGVy","value":"bmV1dHJvbjFtOWwzNTh4dW5oaHdkczA1Njh6YTQ5bXpodnV4eDl1eDh4YWZ4Mg==","index":true},{"key":"YW1vdW50","value":"MzQ3NXN0YWtl","index":true}]},{"type":"message","attributes":[{"key":"c2VuZGVy","value":"bmV1dHJvbjFtOWwzNTh4dW5oaHdkczA1Njh6YTQ5bXpodnV4eDl1eDh4YWZ4Mg==","index":true}]},{"type":"tx","attributes":[{"key":"ZmVl","value":"MzQ3NXN0YWtl","index":true}]},{"type":"tx","attributes":[{"key":"YWNjX3NlcQ==","value":"bmV1dHJvbjFtOWwzNTh4dW5oaHdkczA1Njh6YTQ5bXpodnV4eDl1eDh4YWZ4Mi8xODk=","index":true}]},{"type":"tx","attributes":[{"key":"c2lnbmF0dXJl","value":"OWxPbGw4QmgzMzZoSDRCSFR1enVFODN6T3lNdGRPZ0ZpYmlMd1hhbm1ISXRzWC9RYm9tR0NuOHMzbnUrM2drcmlSK0JFK2NBcUFpOUdGc0E3ZWsrRUE9PQ==","index":true}]},{"type":"message","attributes":[{"key":"YWN0aW9u","value":"L2Nvc213YXNtLndhc20udjEuTXNnSW5zdGFudGlhdGVDb250cmFjdA==","index":true}]},{"type":"message","attributes":[{"key":"bW9kdWxl","value":"d2FzbQ==","index":true},{"key":"c2VuZGVy","value":"bmV1dHJvbjFtOWwzNTh4dW5oaHdkczA1Njh6YTQ5bXpodnV4eDl1eDh4YWZ4Mg==","index":true}]},{"type":"instantiate","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true},{"key":"Y29kZV9pZA==","value":"MTU2","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true},{"key":"YWN0aW9u","value":"aW5zdGFudGlhdGU=","index":true},{"key":"c2VuZGVy","value":"bmV1dHJvbjFtOWwzNTh4dW5oaHdkczA1Njh6YTQ5bXpodnV4eDl1eDh4YWZ4Mg==","index":true}]},{"type":"instantiate","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true},{"key":"Y29kZV9pZA==","value":"MTU3","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true},{"key":"YWN0aW9u","value":"aW5zdGFudGlhdGU=","index":true}]},{"type":"instantiate","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFydTA0Y3IwY3E4d2U4eWpnYXEyODZsODRoOTNjbXUwZTZmaHB3Y3dkNG5wOWxzanEzZWtzZjV0cWEy","index":true},{"key":"Y29kZV9pZA==","value":"MTU4","index":true}]},{"type":"reply","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true},{"key":"Z3JvdXBfY29udHJhY3RfYWRkcmVzcw==","value":"bmV1dHJvbjFydTA0Y3IwY3E4d2U4eWpnYXEyODZsODRoOTNjbXUwZTZmaHB3Y3dkNG5wOWxzanEzZWtzZjV0cWEy","index":true}]},{"type":"execute","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFydTA0Y3IwY3E4d2U4eWpnYXEyODZsODRoOTNjbXUwZTZmaHB3Y3dkNG5wOWxzanEzZWtzZjV0cWEy","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFydTA0Y3IwY3E4d2U4eWpnYXEyODZsODRoOTNjbXUwZTZmaHB3Y3dkNG5wOWxzanEzZWtzZjV0cWEy","index":true},{"key":"YWN0aW9u","value":"YWRkX2hvb2s=","index":true},{"key":"aG9vaw==","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true},{"key":"c2VuZGVy","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true}]},{"type":"execute","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFydTA0Y3IwY3E4d2U4eWpnYXEyODZsODRoOTNjbXUwZTZmaHB3Y3dkNG5wOWxzanEzZWtzZjV0cWEy","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjFydTA0Y3IwY3E4d2U4eWpnYXEyODZsODRoOTNjbXUwZTZmaHB3Y3dkNG5wOWxzanEzZWtzZjV0cWEy","index":true},{"key":"YWN0aW9u","value":"dXBkYXRlX2FkbWlu","index":true},{"key":"YWRtaW4=","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true},{"key":"c2VuZGVy","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true}]},{"type":"reply","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true},{"key":"dm90aW5nX3JlZ3NpdHJ5X21vZHVsZQ==","value":"bmV1dHJvbjFkbmRwcnFqdmU2eXl1bTRtNWRwdGtlbTh6Njc5ZjgyamhzcXBwazlzZHc5d2Y2dHJoNGhxY3docWFh","index":true}]},{"type":"instantiate","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEwcWR0dGw1cW5xZnN1dm04NTJ6cmZ5c3FrdXlkbjBod3o2eGU1NDcydHY1NTkwdjk1emhzc3l4YThh","index":true},{"key":"Y29kZV9pZA==","value":"MTYw","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEwcWR0dGw1cW5xZnN1dm04NTJ6cmZ5c3FrdXlkbjBod3o2eGU1NDcydHY1NTkwdjk1emhzc3l4YThh","index":true},{"key":"YWN0aW9u","value":"aW5zdGFudGlhdGU=","index":true},{"key":"ZGFv","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true}]},{"type":"instantiate","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjF4bThlbXdybnZ1YzU5ZjJtams5Z2Q2cnB6c3I5Zm05anZ0cHZtd3IzY3o0bGNxMjBxamNxdWw1c3U5","index":true},{"key":"Y29kZV9pZA==","value":"MTU5","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjF4bThlbXdybnZ1YzU5ZjJtams5Z2Q2cnB6c3I5Zm05anZ0cHZtd3IzY3o0bGNxMjBxamNxdWw1c3U5","index":true},{"key":"ZGFv","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true},{"key":"ZGVwb3NpdF9pbmZv","value":"U29tZShDaGVja2VkRGVwb3NpdEluZm8geyBkZW5vbTogTmF0aXZlKCJ1bnRybiIpLCBhbW91bnQ6IFVpbnQxMjgoMTApLCByZWZ1bmRfcG9saWN5OiBBbHdheXMgfSk=","index":true},{"key":"bWV0aG9k","value":"aW5zdGFudGlhdGU=","index":true},{"key":"b3Blbl9wcm9wb3NhbF9zdWJtaXNzaW9u","value":"ZmFsc2U=","index":true},{"key":"cHJvcG9zYWxfbW9kdWxl","value":"bmV1dHJvbjEwcWR0dGw1cW5xZnN1dm04NTJ6cmZ5c3FrdXlkbjBod3o2eGU1NDcydHY1NTkwdjk1emhzc3l4YThh","index":true}]},{"type":"reply","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEwcWR0dGw1cW5xZnN1dm04NTJ6cmZ5c3FrdXlkbjBod3o2eGU1NDcydHY1NTkwdjk1emhzc3l4YThh","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEwcWR0dGw1cW5xZnN1dm04NTJ6cmZ5c3FrdXlkbjBod3o2eGU1NDcydHY1NTkwdjk1emhzc3l4YThh","index":true},{"key":"dXBkYXRlX3ByZV9wcm9wb3NlX21vZHVsZQ==","value":"bmV1dHJvbjF4bThlbXdybnZ1YzU5ZjJtams5Z2Q2cnB6c3I5Zm05anZ0cHZtd3IzY3o0bGNxMjBxamNxdWw1c3U5","index":true}]},{"type":"reply","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true}]},{"type":"wasm","attributes":[{"key":"X2NvbnRyYWN0X2FkZHJlc3M=","value":"bmV1dHJvbjEzMHB3amc1ZXJmcjc3bHJlY2xlNzR6NHN3NjJoNnV4YzB2Zm0wOGVheWNsdnd6ejJlc21zNTlmOTc0","index":true},{"key":"cHJvcF9tb2R1bGU=","value":"bmV1dHJvbjEwcWR0dGw1cW5xZnN1dm04NTJ6cmZ5c3FrdXlkbjBod3o2eGU1NDcydHY1NTkwdjk1emhzc3l4YThh","index":true}]}]}

# jq CAN NOT parse the RES value: parse error: Invalid numeric literal at line 1, column 4365
CORE_CONTRACT_ADDR=$(echo $RES | jq -r '.logs[0].events[0].attributes[0].value')
echo "CORE_CONTRACT_ADDR:" $CORE_CONTRACT_ADDRпше