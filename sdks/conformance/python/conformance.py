"""Live conformance: drive the GENERATED Python control-plane client against a
running beaterd and verify typed request/response shapes match the API.

Proves API-shape == SDK-shape for Python. Run via run.sh.
"""

import os
import sys

import beater_client
from beater_client import ApiClient, Configuration
from beater_client.api.health_api import HealthApi
from beater_client.api.traces_api import TracesApi
from beater_client.api.datasets_api import DatasetsApi
from beater_client.models.create_dataset_request import CreateDatasetRequest

BASE = os.environ["BEATER_BASE_URL"].rstrip("/")
TENANT = os.environ.get("BEATER_TENANT", "demo")
PROJECT = os.environ.get("BEATER_PROJECT", "demo")


def main() -> int:
    cfg = Configuration(host=BASE)
    with ApiClient(cfg) as api:
        # 1. health -> typed response
        health = HealthApi(api).health_health()
        assert getattr(health, "ok") is True, f"health.ok != True: {health}"
        print(f"  health: ok={health.ok}")

        # 2. create dataset -> typed request body + typed response (shape parity)
        created = DatasetsApi(api).datasets_create_dataset(
            TENANT, PROJECT, CreateDatasetRequest(name="conformance-py")
        )
        print(f"  createDataset -> {type(created).__name__}")

        # 3. list traces -> typed page response
        page = TracesApi(api).traces_list_traces(TENANT)
        items = getattr(page, "items", None)
        assert items is not None, f"traces.list page missing 'items': {page}"
        print(f"  traces.list -> {type(page).__name__} items={len(items)}")

    print("PASS: python generated client round-trips against live API")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as exc:  # noqa: BLE001
        print(f"FAIL: {type(exc).__name__}: {exc}", file=sys.stderr)
        sys.exit(1)
