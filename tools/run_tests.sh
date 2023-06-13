#!/bin/bash

set -eux

trap 'kill -- -$$' SIGINT SIGABRT EXIT

cargo run > /dev/null &
sleep 2
tox
