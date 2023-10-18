.PHONY: schema test clippy proto-gen build fmt

schema:
	@find contracts/* -type f -name 'Cargo.toml' -execdir cargo schema \;

test:
	@cargo test

clippy:
	@cargo clippy --all --all-targets -- -D warnings

fmt:
	@cargo fmt -- --check

check_contracts:
	@cargo install cosmwasm-check --locked
	@cosmwasm-check --available-capabilities iterator,staking,stargate,neutron artifacts/*.wasm

compile:
	@./build_release.sh

build: schema clippy fmt test compile check_contracts



