BIN=neutrond


CHAIN_ID_1=test-1
CHAIN_ID_2=test-2

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME_1=${NEUTRON_DIR}/data/test-1/
HOME_2=${NEUTRON_DIR}/data/test-2/

USERNAME_1=demowallet1
USERNAME_2=demowallet2
KEY_2=$(neutrond keys show demowallet2 -a --keyring-backend test --home ${HOME_2})
ADMIN=$(neutrond keys show demowallet1 -a --keyring-backend test --home ${HOME_1})


STAKING_ADDRESS=neutron14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s5c2epq

ST_ADRES=14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr

PROPOSE_ADDRESS=neutron18cszlvm6pze0x9sz32qnjq4vtd45xehqs8dq7cwy8yhq35wfnn3q795n8y




TARGET_ADDRESS=neutron1mjk79fjjgpplak5wq838w0yd982gzkyf8fxu8u
VAL2=neutronvaloper1qnk2n4nlkpw9xfqntladh74w6ujtulwnqshepx

# stake funds
RES=$(${BIN} tx wasm execute $ST_ST_ADRES "{\"stake\": {}}" --amount 1000stake --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "staking:"
echo $RES
# propose text proposal
RES=$(${BIN} tx wasm execute $PROPOSE_ADDRESS "{\"propose\": {\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\", [{\"custom\": {\"submit_proposal\": {\"text_proposal\": {\"title\": \"title\",\"description\": \"description\"}}}}]}}" --from ${USERNAME_1} -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo "propose:"
echo $RES
