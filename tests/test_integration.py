import pytest
import httpx
from pathlib import Path
import json

DATA_PATH = Path(__file__).parent / "data"


@pytest.fixture(scope="session")
def client() -> httpx.Client:
    transport = httpx.HTTPTransport(uds="/tmp/cloud-config-validator/sock")
    client = httpx.Client(base_url="http://cloud-config-validator", transport=transport)

    yield client


def test_root_get(client):
    resp = client.get("/")
    assert resp.status_code == 200, resp.content
    assert resp.json() == ['/v1']


def test_x(client):
    resp = client.post(
        "/v1/cloud-config/validate",
        json={
            "format": "yaml",
            "payload": "asdfafd: 1"
        }
    )
    assert resp.status_code == 200, resp.content
    assert resp.json() == {"annotations": [], "errors": [], "is_valid": True}


def get_test_cases(file: Path):
    return [(case["in"], case["out"]["status_code"], case["out"]["json"]) for case in json.loads(file.read_text())]


class TestValidation:

    @pytest.mark.parametrize("input, status_code, expected_json", get_test_cases(DATA_PATH / "test_cases.json"))
    def test_0(self, client, input, status_code, expected_json):
        resp = client.post(
            "/v1/cloud-config/validate",
            json=input,
        )
        assert resp.status_code == status_code, resp
        assert resp.json() == expected_json, resp

    @pytest.mark.parametrize("input, status_code, expected_json", get_test_cases(DATA_PATH / "cloud_init_examples.json"))
    def test_cloud_init_examples(self, client, input, status_code, expected_json):
        resp = client.post(
            "/v1/cloud-config/validate",
            json=input,
        )
        assert resp.status_code == status_code, resp.content
        assert resp.json() == expected_json, resp.content
