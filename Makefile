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
	@docker run --rm -v "$(CURDIR)":/code \
	    --mount type=volume,source="$(notdir $(CURDIR))_cache",target=/target \
	    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	    --platform linux/amd64 \
	    cosmwasm/workspace-optimizer:0.15.0

build: schema clippy fmt test compile check_contracts
