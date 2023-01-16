BIN=neutrond


CHAIN_ID_1=test-1

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME_1=${NEUTRON_DIR}/data/test-1/


USERNAME_1=demowallet1
USERNAME_2=demowallet3
USERNAME_3=rly1

# DAO addresses
VAULT_ADDRESS=neutron14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s5c2epq
VOTE_ADDRESS=neutron1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqqgmq2a
CORE_ADDRESS=neutron1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqcd0mrx
PROPOSE_MULTIPLE_ADDRESS=neutron1pvrwmjuusn9wh34j7y520g8gumuy9xtl3gvprlljfdpwju3x7ucsj3fj40
PRE_PROPOSE_MULTIPLE_ADDRESS=neutron10qt8wg0n7z740ssvf3urmvgtjhxpyp74hxqvqt7z226gykuus7eqjqrsug


#
# send funds to contract to send them back
RES=$(${BIN} tx bank send ${USERNAME_1} ${CORE_ADDRESS}  1000stake  -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)

echo "


bank->send from wallet 1:
"
echo $RES

# STAKING
# stake funds from wallet 1
RES=$(${BIN} tx wasm execute $VAULT_ADDRESS "{\"bond\": {}}" --amount 1000stake --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


staking from wallet 1:
"
echo $RES

#stake funds from wallet 2
RES=$(${BIN} tx wasm execute $VAULT_ADDRESS "{\"bond\": {}}" --amount 1000stake --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


staking from wallet 2:
"
echo $RES

#stake funds from wallet 3
RES=$(${BIN} tx wasm execute $VAULT_ADDRESS "{\"bond\": {}}" --amount 1000stake --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


staking from wallet 3:
"
echo $RES


# PROPOSAL 1 (to pick 1 option)
#propose proposal we're going to pass
PARAM_CHANGES="[{\"subspace\":\"icahost\",\"key\":\"HostEnabled\",\"value\":\"false\"}]"
PARAM_CHANGE_PROPOSAL="{\"param_change_proposal\":{\"title\":\"title\",\"description\":\"description\",\"param_changes\":$PARAM_CHANGES}}"
PARAM_CHANGES_2="[{\"subspace\":\"icahost\",\"key\":\"HostEnabled\",\"value\":\"true\"}]"
PARAM_CHANGE_PROPOSAL_2="{\"param_change_proposal\":{\"title\":\"title2\",\"description\":\"description2\",\"param_changes\":$PARAM_CHANGES_2}}"
MSGS="[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\": $PARAM_CHANGE_PROPOSAL}}}]"
MSGS_2="[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\": $PARAM_CHANGE_PROPOSAL_2}}}]"
CHOICES="{\"options\": [{\"description\": \"choice1\", \"msgs\": $MSGS}, {\"description\": \"choice2\", \"msgs\": $MSGS_2}]}"
PROPOSE="{\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", \"choices\": $CHOICES}"
PROP="{\"propose\": {\"msg\": {\"propose\": $PROPOSE}}}"

RES=$(${BIN} tx wasm execute $PRE_PROPOSE_MULTIPLE_ADDRESS "$PROP" --amount 1000stake --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "

propose proposal to be passed:
"
echo $RES

#### vote option 1 from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\": {\"option_id\": 1}}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote option 1 from wallet1:
"
echo $RES

#### vote option 0 from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\": {\"option_id\": 0}}}" --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote option 0 from wallet 2:
"
echo $RES

#### vote option 1 from wallet 3
RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"vote\": {\"proposal_id\": 1, \"vote\": {\"option_id\": 1}}}" --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote option 1 from wallet 3:
"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"execute\": {\"proposal_id\": 1}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


execute proposal:
"
echo $RES






# PROPOSAL 2 (to pick "none of the above" option)
#propose proposal we're going to pass
PARAM_CHANGES="[{\"subspace\":\"icahost\",\"key\":\"HostEnabled\",\"value\":\"false\"}]"
PARAM_CHANGE_PROPOSAL="{\"param_change_proposal\":{\"title\":\"title\",\"description\":\"description\",\"param_changes\":$PARAM_CHANGES}}"
PARAM_CHANGES_2="[{\"subspace\":\"icahost\",\"key\":\"HostEnabled\",\"value\":\"true\"}]"
PARAM_CHANGE_PROPOSAL_2="{\"param_change_proposal\":{\"title\":\"title2\",\"description\":\"description2\",\"param_changes\":$PARAM_CHANGES_2}}"
MSGS="[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\": $PARAM_CHANGE_PROPOSAL}}}]"
MSGS_2="[{\"custom\":{\"submit_admin_proposal\":{\"admin_proposal\": $PARAM_CHANGE_PROPOSAL_2}}}]"
CHOICES="{\"options\": [{\"description\": \"choice1\", \"msgs\": $MSGS}, {\"description\": \"choice2\", \"msgs\": $MSGS_2}]}"
PROPOSE="{\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", \"choices\": $CHOICES}"
PROP="{\"propose\": {\"msg\": {\"propose\": $PROPOSE}}}"

RES=$(${BIN} tx wasm execute $PRE_PROPOSE_MULTIPLE_ADDRESS "$PROP" --amount 1000stake --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "

propose proposal to be not passed:
"
echo $RES

#### vote option 2 from wallet 1
RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"vote\": {\"proposal_id\": 2, \"vote\": {\"option_id\": 2}}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote option 2 from wallet1:
"
echo $RES

#### vote option 0 from wallet 2
RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"vote\": {\"proposal_id\": 2, \"vote\": {\"option_id\": 0}}}" --from ${USERNAME_2} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote option 0 from wallet 2:
"
echo $RES

#### vote option 2 from wallet 3
RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"vote\": {\"proposal_id\": 2, \"vote\": {\"option_id\": 2}}}" --from ${USERNAME_3} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


vote option 2 from wallet 3:
"
echo $RES

RES=$(${BIN} tx wasm execute $PROPOSE_MULTIPLE_ADDRESS "{\"execute\": {\"proposal_id\": 2}}"  --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


execute proposal (should fail because not passed):
"
echo $RES



RES=$(${BIN} q wasm contract-state smart  $CORE_ADDRESS "{\"total_power_at_height\": {}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "


total voting power, should be 3000:
"
echo $RES

RES=$(${BIN} q wasm contract-state smart  $PROPOSE_MULTIPLE_ADDRESS "{\"proposal\": {\"proposal_id\": 1}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "
should be status:executed"
echo $RES

RES=$(${BIN} q wasm contract-state smart  $PROPOSE_MULTIPLE_ADDRESS "{\"proposal\": {\"proposal_id\": 2}}"  --chain-id ${CHAIN_ID_1} --output json  --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "
should be status:rejected"
echo $RES

sleep 5

echo "
list archived adminmodule proposals (should still be one param change proposal (first choice of proposal#1))"
RES=$(${BIN} q adminmodule archivedproposals --chain-id ${CHAIN_ID_1} --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo $RES
