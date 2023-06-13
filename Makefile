CARGO=cargo
SNAP_NAME=cloud-config-validator

all: lint

clean:
	$(CARGO) clean

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
