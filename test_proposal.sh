BIN=neutrond


CHAIN_ID_1=test-1

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME_1=${NEUTRON_DIR}/data/test-1/


USERNAME_1=demowallet1
USERNAME_2=demowallet3
USERNAME_3=rly1

# DAO addresses
VAULT_ADDRESS=neutron14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s5c2epq
PROPOSE_ADDRESS=neutron1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qmysp02
VOTE_ADDRESS=neutron1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqqgmq2a
CORE_ADDRESS=neutron1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqcd0mrx
PRE_PROPOSE_ADDRESS=neutron1eyfccmjm6732k7wp4p6gdjwhxjwsvje44j0hfx8nkgrm8fs7vqfs8hrpdj


#
# send funds to contract to send them back
RES=$(${BIN} tx bank send ${USERNAME_1} ${CORE_ADDRESS}  1000untrn  -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)

echo "


bank->send from wallet 1:
"
echo $RES

# STAKING
# bond funds from wallet 1
RES=$(${BIN} tx wasm execute $VAULT_ADDRESS "{\"bond\": {}}" --amount 1000untrn --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


staking from wallet 1:
"
echo $RES

# bond funds from wallet 2
RES=$(${BIN} tx wasm execute $VAULT_ADDRESS "{\"bond\": {}}" --amount 1000untrn --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


staking from wallet 2:
"
echo $RES

# bond funds from wallet 3
RES=$(${BIN} tx wasm execute $VAULT_ADDRESS "{\"bond\": {}}" --amount 1000untrn --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


staking from wallet 3:
"
echo $RES



PROP="{\"propose\":  { \"msg\": { \"propose\": {\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", \"msgs\":[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\":{\"param_change_proposal\":{\"title\":\"title\",\"description\":\"description\",\"param_changes\":[{\"subspace\":\"icahost\",\"key\":\"HostEnabled\",\"value\":\"false\"}]}}}}}]}}}}"
# PROPOSAL 1 (to pass)
#propose proposal we're going to pass
RES=$(${BIN} tx wasm execute $PRE_PROPOSE_ADDRESS "$PROP" --amount 1000untrn --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


proposing a proposal:
"
echo $RES

echo "


propose proposal to be passed:
"
echo $RES

#### vote YES from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\":  \"yes\"}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet1:
"
echo $RES

#### vote NO from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\":  \"no\"}}"  --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote NO from wallet 2:
"
echo $RES

#### vote YES from wallet 3
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\":  \"yes\"}}"  --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet 3:
"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"execute\": {\"proposal_id\": 1}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


execute proposal:
"
echo $RES



# PROPOSAL 2 (to decline)
#propose proposal we're going to pass
RES=$(${BIN} tx wasm execute $PRE_PROPOSE_ADDRESS "{\"propose\":  { \"msg\": { \"propose\": {\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", \"msgs\":[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\":{\"param_change_proposal\":{\"title\":\"title\",\"description\":\"description\",\"param_changes\":[{\"subspace\":\"icahost\",\"key\":\"HostEnabled\",\"value\":\"false\"}]}}}}}]}}}}" --amount 1000untrn --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo ".
.
.
propose proposal to be decline:

"
echo $RES

#### vote YES from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 2, \"vote\":  \"yes\"}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo  ".
.
.
vote YES from wallet1:
"
echo $RES

#### vote NO from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 2, \"vote\":  \"no\"}}"  --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote NO from wallet 2:
"
echo $RES

#### vote NO from wallet 3
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 2, \"vote\":  \"no\"}}"  --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote NO from wallet 3:
"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"execute\": {\"proposal_id\": 2}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


execute proposal: should fail
"
echo $RES




# PROPOSAL 3 (to pass)
#propose proposal we're going to pass
RES=$(${BIN} tx wasm execute $PRE_PROPOSE_ADDRESS "{\"propose\":  { \"msg\": {\"propose\": {\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", \"msgs\":[{\"bank\":{\"send\":{\"to_address\":\"neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2\",\"amount\":[{\"denom\":\"untrn\",\"amount\":\"1000\"}]}}}]}}}}" --amount 1000untrn --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


propose proposal # 3 to be passed:
"
echo $RES

#### vote YES from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 3, \"vote\":  \"yes\"}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet1:
"
echo $RES

#### vote NO from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 3, \"vote\":  \"no\"}}"  --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote NO from wallet 2:
"
echo $RES

#### vote YES from wallet 3
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 3, \"vote\":  \"yes\"}}"  --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet 3:
"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"execute\": {\"proposal_id\": 3}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


execute proposal:
"
echo $RES



# PROPOSAL 4: try submit and execute SoftwareUpgradeProposal (should pass)

PROP="{\"propose\":  { \"msg\": { \"propose\": {\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", \"msgs\":[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\":{\"software_upgrade_proposal\":{\"title\":\"title\",\"description\":\"description\",\"plan\":{\"name\":\"planname#1\",\"height\":10000,\"info\":\"asdfasdfasdf\"}}}}}}]}}}}"
# PROPOSAL 4 (to pass) (other type)
#propose proposal we're going to pass
RES=$(${BIN} tx wasm execute $PRE_PROPOSE_ADDRESS "$PROP" --amount 1000untrn --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


propose proposal to be passed:
"
echo $RES

#### vote YES from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 4, \"vote\":  \"yes\"}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet1:
"
echo $RES

#### vote NO from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 4, \"vote\":  \"no\"}}"  --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote NO from wallet 2:
"
echo $RES

#### vote YES from wallet 3
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 4, \"vote\":  \"yes\"}}"  --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet 3:
"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"execute\": {\"proposal_id\": 4}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


execute proposal 4:
"
echo $RES



# Total voting power and check statuses

RES=$(${BIN} q wasm contract-state smart  $CORE_ADDRESS "{\"total_power_at_height\": {}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


total voting power, should be 3000:
"
echo $RES

RES=$(${BIN} q wasm contract-state smart  $PROPOSE_ADDRESS "{\"proposal\": {\"proposal_id\": 1}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "
should be status:executed"
echo $RES

RES=$(${BIN} q wasm contract-state smart  $PROPOSE_ADDRESS "{\"proposal\": {\"proposal_id\": 2}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "
should be status:rejected"
echo $RES

RES=$(${BIN} q wasm contract-state smart  $PROPOSE_ADDRESS "{\"proposal\": {\"proposal_id\": 3}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "
should be status:executed"
echo $RES

RES=$(${BIN} q wasm contract-state smart  $PROPOSE_ADDRESS "{\"proposal\": {\"proposal_id\": 4}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "
should be status:executed"
echo $RES
sleep 5

echo "
list archived adminmodule proposals (should be one param change proposal and one software upgrade proposal)"
RES=$(${BIN} q adminmodule archivedproposals   --chain-id ${CHAIN_ID_1}   --home ${HOME_1}   --node tcp://127.0.0.1:16657)
echo $RES


# PROPOSAL 5: try submit and execute CancelSoftwareUpgradeProposal (should pass)

PROP="{\"propose\":  { \"msg\": { \"propose\": {\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", \"msgs\":[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\":{\"cancel_software_upgrade_proposal\":{\"title\":\"title\",\"description\":\"description\"}}}}}]}}}}"
# PROPOSAL 5 (to pass) (other type)
#propose proposal we're going to pass
RES=$(${BIN} tx wasm execute $PRE_PROPOSE_ADDRESS "$PROP" --amount 1000untrn --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


propose proposal to be passed:
"
echo $RES

#### vote YES from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 5, \"vote\":  \"yes\"}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet1:
"
echo $RES

#### vote NO from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 5, \"vote\":  \"no\"}}"  --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote NO from wallet 2:
"
echo $RES

#### vote YES from wallet 3
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"vote\": {\"proposal_id\": 5, \"vote\":  \"yes\"}}"  --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote YES from wallet 3:
"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"execute\": {\"proposal_id\": 5}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025untrn --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


execute proposal:
"
echo $RES

echo "
list archived adminmodule proposals (should be one param change proposal and one software upgrade proposal and one cancel software upgrade proposal)"
RES=$(${BIN} q adminmodule archivedproposals   --chain-id ${CHAIN_ID_1}   --home ${HOME_1}   --node tcp://127.0.0.1:16657)
echo $RES
