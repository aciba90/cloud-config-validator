#!/bin/bash

set -eux

trap 'kill -- -$$' SIGINT SIGABRT EXIT

cargo run --bin server > /dev/null &
sleep 2
tox
