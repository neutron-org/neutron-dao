.PHONY: schema test clippy proto-gen build fmt

schema:
	@find contracts/* -maxdepth 1 -type d \( ! -name . \) -exec bash -c "cd '{}' && cargo schema" \;

test:
	@cargo test

clippy:
	@cargo clippy --all --all-targets -- -D warnings

fmt:
	@cargo fmt -- --check

check_contracts:
	@cargo install cosmwasm-check
	@cosmwasm-check --available-capabilities iterator,staking,stargate,neutron artifacts/*.wasm

compile:
	@./build_release.sh

build: schema clippy fmt compile check_contracts



