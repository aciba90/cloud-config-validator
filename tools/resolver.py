from pathlib import Path
import json

SCHEMA_INPUT = Path(__file__).parent.parent / "schemas" / "versions.schema.cloud-config.resolved.json"


def resolve_references(data, defs):
    if isinstance(data, dict):
        if "$ref" in data:
            # print(data["$ref"])
            ref = data["$ref"].split("#/$defs/")[-1]
            data = defs[ref]
        new = {}
        for k, v in data.items():
            new[k] = resolve_references(v, defs)
        return new
    if isinstance(data, (list, tuple)):
        new = []
        for d in data:
            new.append(resolve_references(d, defs))
        return new
    return data


def main():
    schema = json.loads(SCHEMA_INPUT.read_text())
    defs = schema.pop("$defs")
    new_schema = resolve_references(schema, defs)
    print(json.dumps(new_schema))


if __name__ == "__main__":
    main()
