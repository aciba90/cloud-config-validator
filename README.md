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
