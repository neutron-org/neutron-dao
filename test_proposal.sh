BIN=neutrond

CONTRACT_ADDRESS=neutron14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s5c2epq

CHAIN_ID_1=test-1
CHAIN_ID_2=test-2

NEUTRON_DIR=${NEUTRON_DIR:-../neutron}
HOME_1=${NEUTRON_DIR}/data/test-1/
HOME_2=${NEUTRON_DIR}/data/test-2/

USERNAME_1=demowallet1
USERNAME_2=demowallet2
KEY_2=$(neutrond keys show demowallet2 -a --keyring-backend test --home ${HOME_2})
ADMIN=$(neutrond keys show demowallet1 -a --keyring-backend test --home ${HOME_1})

TARGET_ADDRESS=neutron1mjk79fjjgpplak5wq838w0yd982gzkyf8fxu8u
VAL2=neutronvaloper1qnk2n4nlkpw9xfqntladh74w6ujtulwnqshepx

# Submit a proposal
RES=$(${BIN} tx wasm execute $CONTRACT_ADDRESS "{\"submit_text_proposal\": {\"title\": \"TEST\", \"description\": \"BOTTOMTTEXT\"}}" --from ${USERNAME_1}  -y --chain-id ${CHAIN_ID_1} --output json --broadcast-mode=block --gas-prices 0.0025stake --gas 1000000 --keyring-backend test --home ${HOME_1} --node tcp://127.0.0.1:16657)
echo $RES

sleep 5
# print proposal
echo "--- q gov proposals"
RES=$(${BIN} query adminmodule archivedproposals   --chain-id ${CHAIN_ID_1}   --home ${HOME_1}   --node tcp://127.0.0.1:16657)
echo $RES


