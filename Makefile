integration_tests:
	( (cargo run --bin local; sleep 2)& tox)
