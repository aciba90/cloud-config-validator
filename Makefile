CARGO=cargo

all: build

build:
	snapcraft --debug

install:
	snap install --dangerous cloud-config-validator-test*.snap
	@snap restart cloud-config-validator-test.daemon
	snap services cloud-config-validator-test

clean:
	$(CARGO) clean
	rm -rf cloud-config-validator*.snap

integration_tests:
	( (cargo run --bin local; sleep 2)& tox)

lint: check fmt clippy

check:
	$(CARGO) check --all-targets

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

fmt:
	$(CARGO) fmt --all -- --check
