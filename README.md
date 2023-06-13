# Cloud Config Validator

## Architecture

[Axum](https://github.com/tokio-rs/axum) server handling requests asynchronously with green threads over a multi-threaded runtime.

The validation endpoints execute the validation step (which is a blocking CPU-bound computation) within a separated [rayon](https://docs.rs/rayon/latest/rayon/) thread pool optimized for CPU-bound computations.

The jsonschema is parsed at start time and refreshed periodically. And it is shared across threads.

## Examples

```sh
$ sudo curl --unix-socket /var/snap/cloud-config-validator/common/unix.socket \
	-v -X POST http://0.0.0.0/v1/cloud-config/validate -H "Content-Type: application/json" \
	-d '{"payload": "#cloud-config\nubuntu_advantage:\n  features:\n    disable_auto_attach: 1"}' | jq
```

```json
{
  "annotations": [],
  "errors": [
    {
      "description": "1 is not of type \"boolean\"",
      "instance_path": "/ubuntu_advantage/features/disable_auto_attach"
    }
  ],
  "is_valid": false
}
```

```sh
$ sudo curl --unix-socket /var/snap/cloud-config-validator/common/unix.socket \
	-v -X POST http://0.0.0.0/v1/cloud-config/validate -H "Content-Type: application/json" \
	-d '{"payload": "#cloud-config\nusers:\n  - name: a\n    uid: \"1743\"" }' | jq
```

```json
{
  "annotations": [
    {
      "description": "DEPRECATED: The use of ``string`` type will be dropped after April 2027. Use an ``integer`` instead.",
      "instance_path": "/users/0/uid"
    }
  ],
  "errors": [],
  "is_valid": true
}
```
