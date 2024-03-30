# ccv server

## Build, run and test

To run it locally execute `cargo run --release`.

To build and run it as a docker container execute:

```sh
docker build -t <IMAGE_NAME>:<TAG> .
docker run -p 3000:3000 <IMAGE_NAME>:<TAG>
```

Docker images are published as part of the CI/CD workflow. To pull and image run:

```sh
docker pull ghcr.io/aciba90/cloud-config-validator:main
```

To execute the unit test run `cargo test` and to run the integration tests run `tox`.

## API

<details>
  <summary>
    <code>POST</code> <code><b>/v1/{cloud-config,network-config}/validate</b></code>
    <code> Validates a cloud-config or a network-config, responding with JSONPointers pointing to errors and deprecations</code>
  </summary>

  ### Parameters

  > | name      |  type     | data type               | description                                                           |
  > |-----------|-----------|-------------------------|-----------------------------------------------------------------------|
  > | None      |  required | JSON                    | See [Request body format](#request-body-format)                       |

  ### Responses

  > | http code     | content-type                      | response                                                            |
  > |---------------|-----------------------------------|---------------------------------------------------------------------|
  > | `200`         | `application/json`                | See [Response body format](#response-body-format)                   |

</details>

### Request body format

```json
{
  "payload": "<cloud-config or networ-config>"
}
```

### Response body format

```json
{
  "annotations": [
    {
      "description": "<description>",
      "instance_path": "<JSONPointer>"
    }
  ],
  "errors": [
    {
      "description": "<description>",
      "instance_path": "<JSONPointer>"
    }
  ],
  "is_valid":true
}
```

### Examples

```sh
docker run -p 3000:3000 ghcr.io/aciba90/cloud-config-validator:main

$ curl http://0.0.0.0:3000/v1/cloud-config/validate -H "Content-Type: application/json" -d '{"payload": "#cloud-config\nubuntu_advantage:\n  features:\n    disable_auto_attach: 1"}'
{"annotations":[],"errors":[{"description":"1 is not of type \"boolean\"","instance_path":"/ubuntu_advantage/features/disable_auto_attach"}],"is_valid":false}

$ curl http://0.0.0.0:3000/v1/cloud-config/validate -H "Content-Type: application/json" -d '{"payload": "#cloud-config\nusers:\n  - name: a\n    uid: \"1743\"" }'
{"annotations":[{"description":"Changed in version 22.3. The use of ``string`` type is deprecated. Use an ``integer`` instead.","instance_path":"/users/0/uid"}],"errors":[],"is_valid":true}
```

## Architecture

[Axum](https://github.com/tokio-rs/axum) server handling requests asynchronously with green threads over a multi-threaded runtime.

The validation endpoints execute the validation step (which is a blocking CPU-bound computation) within a separated [rayon](https://docs.rs/rayon/latest/rayon/) thread pool optimized for CPU-bound computations.

The jsonschema is parsed at start time and refreshed periodically. And it is shared across threads.
