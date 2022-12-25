# Cloud Config Validator

## Installation

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

- OpenAPI doc generation.

  - https://docs.rs/aide/latest/aide/
  - https://github.com/tokio-rs/axum/issues/50

- Async jsonschema fetcher / resolver.
jsonschema-rs' one uses a reqwest::blocking client which provokes
runtime within runtime issues.

- Mechanism to refresh the current validator if a new schema is published.
An initial version could just mean to restart the application, but long term
we need to be able to hot-update the schema without interupting the service.

- Return markers for annotations & errors: https://docs.rs/yaml-peg/latest/yaml_peg/

- Port integration tests to rust.

- snap: integration test
- snap: add slot with the socket
- snap: add to CI
