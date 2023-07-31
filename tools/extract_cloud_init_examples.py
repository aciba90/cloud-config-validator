#!/bin/env python3
import argparse
import importlib
import json
import sys
from pathlib import Path

OUT_PATH = Path(__file__).parent.parent / "tests" / "data" / "cloud_init_examples.json"


def extract_cloud_init_examples(cloud_init_path):
    sys.path.insert(0, cloud_init_path)
    examples = importlib.import_module(
        "tests.unittests.config.test_modules"
    ).get_modules()
    examples = list(map(lambda e: (e.id, e.values[-1]), examples))
    examples = sorted(examples, key=lambda x: x[0])
    examples = list(
        map(
            lambda e: {
                "name": e[0],
                "in": {"payload": e[1]},
                "out": {
                    "status_code": 200,
                    "json": {"annotations": [], "errors": [], "is_valid": True},
                },
            },
            examples,
        )
    )
    return examples


def write_examples(examples):
    with open(OUT_PATH, "w") as f:
        json.dump(examples, f, sort_keys=True, indent=2)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Extract cloud-init module examples.",
    )
    parser.add_argument("cloud_init_path")
    args = parser.parse_args()

    examples = extract_cloud_init_examples(args.cloud_init_path)
    write_examples(examples)
