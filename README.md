# Cloud Config Validator

## TODO

- Error handling. ApiError: encode format with type.

- OpenAPI doc generation.

- Mechanism to refresh the current validator if a new schema is published.
An initial version could just mean to restart the application, but long term
we need to be able to hot-update the schema without interupting the service.

- Async jsonschema fetcher / resolver.
jsonschema-rs' one uses a reqwest::blocking client which provokes
runtime within runtime issues.

- Snap with UDS entry-point.

