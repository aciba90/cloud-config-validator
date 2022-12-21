all: build

integration_tests:
	( (cargo run --bin local; sleep 2)& tox)

build:
	snapcraft --debug

install:
	snap install --dangerous cloud-config-validator-test*.snap
	@snap restart cloud-config-validator-test.daemon
	snap services cloud-config-validator-test

clean:
	cargo clean
	rm -rf cloud-config-validator*.snap
