CARGO=cargo
SNAP_NAME=cloud-config-validator

all: build

build:
	snapcraft --debug

install:
	snap install --dangerous &(SNAP_NAME)*.snap
	@snap restart cloud-config-validator-test.daemon
	snap services cloud-config-validator-test

clean:
	$(CARGO) clean
	rm -rf cloud-config-validator*.snap

integration_tests:
	( (cargo run --bin local; sleep 2)& tox)

test_snap:
	sudo CCV_SOCKET="/var/snap/$(SNAP_NAME)/common/unix.socket" tox

lint: check fmt clippy

check:
	$(CARGO) check --all-targets

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

fmt:
	$(CARGO) fmt --all -- --check

fix: fix-fmt git-add fix-clippy

fix-fmt:
	$(CARGO) fmt --all

fix-clippy:
	$(CARGO) clippy --all-targets --fix --allow-staged

git-add:
	git add .
