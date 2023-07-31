import json
import logging
import os
from pathlib import Path

import httpx
import pytest

DATA_PATH = Path(__file__).parent / "data"
logger = logging.getLogger(__file__)


@pytest.fixture(scope="session")
def client() -> httpx.Client:
    host = os.getenv("CCV_HOST", "0.0.0.0")
    port = os.getenv("CCV_PORT", "3000")
    logger.info("Testing against %s:%s", host, port)

    client = httpx.Client(base_url=f"http://{host}:{port}")

    yield client


def test_root_get(client):
    resp = client.get("/")
    assert resp.status_code == 200, resp.content
    assert resp.json() == ["/v1"]


def test_x(client):
    resp = client.post(
        "/v1/cloud-config/validate",
        json={"format": "yaml", "payload": "#cloud-config\nasdfafd: 1"},
    )
    assert resp.status_code == 200, resp.content
    assert resp.json() == {"annotations": [], "errors": [], "is_valid": True}


def get_test_cases(file: Path):
    return [
        pytest.param(
            case["in"], case["out"]["status_code"], case["out"]["json"], id=case["name"]
        )
        for case in json.loads(file.read_text())
    ]


class TestValidation:
    @pytest.mark.parametrize(
        "input, status_code, expected_json",
        get_test_cases(DATA_PATH / "test_cases.json"),
    )
    def test_0(self, client, input, status_code, expected_json):
        resp = client.post(
            "/v1/cloud-config/validate",
            json=input,
        )
        assert resp.status_code == status_code, resp
        assert resp.json() == expected_json, resp

    @pytest.mark.parametrize(
        "input, status_code, expected_json",
        get_test_cases(DATA_PATH / "cloud_init_examples.json"),
    )
    def test_cloud_init_examples(self, client, input, status_code, expected_json):
        input["payload"] = f"#cloud-config\n{input['payload']}"
        resp = client.post(
            "/v1/cloud-config/validate",
            json=input,
        )
        assert resp.status_code == status_code, resp.content
        assert resp.json() == expected_json, resp.content
