CARGO=cargo
SNAP_NAME=cloud-config-validator

all: build

build:
	snapcraft --debug

install:
	snap install --dangerous $(SNAP_NAME)*.snap
	@snap restart $(SNAP_NAME).daemon
	snap services $(SNAP_NAME)

clean:
	$(CARGO) clean
	rm -rf cloud-config-validator*.snap

integration_tests:
	bash -c "trap 'kill -- -$$$$' EXIT; echo $$$$; sleep 1"
	# (cargo run --bin server &); \
	# sleep 2; tox

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
