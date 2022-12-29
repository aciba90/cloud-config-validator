# Cloud Config Validator

## Installation

To build and install the local validator as snap:

```sh
$ make && sudo make install
```

## Example

```sh
$ sudo curl --unix-socket /var/snap/cloud-config-validator/common/unix.socket -v -X POST http://0.0.0.0:3000/v1/cloud-config/validate -H "Content-Type: application/json" -d '{"payload": "{\"ubuntu_advantage\": {\"features\": { \"disable_auto_attach\": 1}}}", "format": "json"}' | jq .
{
  "annotations": [],
  "errors": [
    {
      "description": "Cloud-config needs to begin with \"#cloud-config\"",
      "instance_path": ""
    },
    {
      "description": "1 is not of type \"boolean\"",
      "instance_path": "/ubuntu_advantage/features/disable_auto_attach"
    }
  ],
  "is_valid": false
}
```

## TODO

- Error handling. ApiError: encode format with type.

- local/snap: logging to journald and file

- OpenAPI doc generation.

  - https://docs.rs/aide/latest/aide/
  - https://github.com/tokio-rs/axum/issues/50

- Return markers for annotations & errors: https://docs.rs/yaml-peg/latest/yaml_peg/

- Port integration tests to rust.

- snap: integration test
- snap: add slot with the socket
- snap: add to CI

- clean-up: remove vendored jsonschema
- unit test coverage
