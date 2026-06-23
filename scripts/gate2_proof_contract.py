import hashlib
import re
from dataclasses import dataclass


REMOTE_URL = "https://github.com/jadenfix/beater.git"
REMOTE_MAIN_REF = "refs/heads/main"
RAW_PREFLIGHT_PATH = "scripts/gate2-outside-local-preflight.sh"
RAW_PREFLIGHT_URL_PREFIX = "https://raw.githubusercontent.com/jadenfix/beater"
GATE2_GHCR_OWNER_REPO = "jadenfix/beater"
GATE2_GHCR_PREFIX = f"ghcr.io/{GATE2_GHCR_OWNER_REPO}"
GATE2_EXPECTED_PLATFORMS = ["linux/amd64", "linux/arm64"]
DEFAULT_API_ENDPOINT = "http://127.0.0.1:8080"
DEFAULT_DASHBOARD_BASE = "http://127.0.0.1:3000"
DEFAULT_OTLP_ENDPOINT = "http://127.0.0.1:4317"
GATE2_FULL_RUN_PORTS = [
    (8080, "beaterd HTTP", "BEATER_HTTP_PORT"),
    (4317, "OTLP gRPC", "BEATER_OTLP_GRPC_PORT"),
    (3000, "dashboard", "BEATER_DASHBOARD_PORT"),
]
GATE2_OUTSIDE_ENV_NAMES = [
    "BEATER_GATE2_OUTSIDE_RUN_DRY_RUN",
    "BEATER_GATE2_EXPECTED_ORIGIN",
    "BEATER_GATE2_OUTSIDE_WRAPPER",
    "BEATER_GATE2_CLONE_STARTED_EPOCH",
    "BEATER_DASHBOARD_PORT",
    "BEATER_HTTP_PORT",
    "BEATER_OTLP_GRPC_PORT",
    "BEATER_GATE2_REUSE",
    "BEATER_GATE2_LOCAL_BUILD",
    "BEATER_GATE2_PULL_POLICY",
    "BEATER_GATE2_WRITE_PROOF",
    "BEATER_GATE2_BROWSER_PROOF",
    "BEATER_GATE2_RECORD_DEMO",
    "BEATER_GATE2_POST_SLO_TIMEOUT_SECONDS",
    "BEATER_GATE2_RUN_ID",
    "BEATER_GATE2_CONFIRMATION_SALT",
    "BEATER_GATE2_REGISTRY_FIXTURE_UNSAFE_FOR_TESTS",
    "BEATERD_IMAGE",
    "BEATER_DASHBOARD_IMAGE",
    "BEATER_DASHBOARD_E2E_IMAGE",
    "BEATER_OTEL_PYTHON_IMAGE",
    "BEATER_GATE2_STOPWATCH_PROOF",
    "BEATER_GATE2_RECORD_VIDEO",
    "BEATER_GATE2_RECORD_NOTES",
    "BEATER_GATE2_COMPOSE_LOGS",
    "BEATER_GATE2_TERMINAL_LOG",
    "KEEP_BEATER_COMPOSE",
    "COMPOSE_FILE",
    "COMPOSE_PROJECT_NAME",
    "COMPOSE_PROFILES",
    "BEATER_GATE2_FIXTURE_FULL_RUN",
]
GATE2_OUTSIDE_ENV_PREFIXES = ["GIT_CONFIG_"]
GATE2_CONFIRMATION_HASH_PREFIX = "gate2"
GATE2_CONFIRMATION_TEST_VECTOR = {
    "salt": "gate2-contract-test-salt",
    "trace_id": "0123456789abcdef0123456789abcdef",
    "span_id": "0123456789abcdef",
    "code": "AB743641",
}
PUBLIC_GIT_ENV = (
    "GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null "
    "GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_COUNT=0"
)


@dataclass(frozen=True)
class Gate2Image:
    image_name: str
    service: str
    env_var: str
    proof_ref_field: str
    proof_digest_field: str

    @property
    def repo(self):
        return f"{GATE2_GHCR_PREFIX}/{self.image_name}"

    @property
    def registry_repository(self):
        return f"{GATE2_GHCR_OWNER_REPO}/{self.image_name}"


GATE2_IMAGES = [
    Gate2Image(
        image_name="beaterd",
        service="beaterd",
        env_var="BEATERD_IMAGE",
        proof_ref_field="Beater image reference",
        proof_digest_field="Beater image digest",
    ),
    Gate2Image(
        image_name="dashboard",
        service="dashboard",
        env_var="BEATER_DASHBOARD_IMAGE",
        proof_ref_field="Dashboard image reference",
        proof_digest_field="Dashboard image digest",
    ),
    Gate2Image(
        image_name="dashboard-e2e",
        service="dashboard-e2e",
        env_var="BEATER_DASHBOARD_E2E_IMAGE",
        proof_ref_field="Dashboard e2e image reference",
        proof_digest_field="Dashboard e2e image digest",
    ),
    Gate2Image(
        image_name="otel-python",
        service="otel-python",
        env_var="BEATER_OTEL_PYTHON_IMAGE",
        proof_ref_field="OTEL Python image reference",
        proof_digest_field="OTEL Python image digest",
    ),
]
GATE2_IMAGE_NAMES = [image.image_name for image in GATE2_IMAGES]
GATE2_IMAGE_BY_NAME = {image.image_name: image for image in GATE2_IMAGES}


def gate2_image(image_name):
    try:
        return GATE2_IMAGE_BY_NAME[image_name]
    except KeyError as err:
        raise KeyError(f"unknown Gate 2 image: {image_name}") from err


def gate2_image_repo(image_name):
    return gate2_image(image_name).repo


def gate2_registry_repository(image_name):
    return gate2_image(image_name).registry_repository


def gate2_image_ref(image_name, tag):
    return f"{gate2_image_repo(image_name)}:{tag}"


def gate2_image_digest_prefix(image_name):
    return f"{gate2_image_repo(image_name)}@sha256:"


def gate2_confirmation_code(salt, trace_id, span_id):
    payload = f"{GATE2_CONFIRMATION_HASH_PREFIX}:{salt}:{trace_id}:{span_id}"
    return hashlib.sha256(payload.encode()).hexdigest()[:8].upper()


assert (
    gate2_confirmation_code(
        GATE2_CONFIRMATION_TEST_VECTOR["salt"],
        GATE2_CONFIRMATION_TEST_VECTOR["trace_id"],
        GATE2_CONFIRMATION_TEST_VECTOR["span_id"],
    )
    == GATE2_CONFIRMATION_TEST_VECTOR["code"]
)
PUBLIC_SHA_RESOLUTION_COMMAND = (
    f'sha_line="$({PUBLIC_GIT_ENV} git ls-remote --exit-code {REMOTE_URL} {REMOTE_MAIN_REF})" && '
    'sha="${sha_line%%[[:space:]]*}" && test -n "$sha"'
)
RAW_PUBLIC_PREFLIGHT_COMMAND = (
    'preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && '
    f'curl -fsSL "{RAW_PREFLIGHT_URL_PREFIX}/$sha/{RAW_PREFLIGHT_PATH}" '
    '-o "$preflight" && BEATER_GATE2_EXPECTED_COMMIT="$sha" bash "$preflight"'
)
CLONE_VERIFICATION_COMMAND = (
    f"{PUBLIC_GIT_ENV} git clone {REMOTE_URL} && cd ./beater && "
    f'test "$({PUBLIC_GIT_ENV} git rev-parse HEAD)" = "$sha" && '
    f'BEATER_GATE2_CLONE_STARTED_EPOCH="$t" {PUBLIC_GIT_ENV} '
    "scripts/gate2-outside-run.sh"
)
OUTSIDE_RUNNER_COMMAND = (
    f"bash -o pipefail -lc '{PUBLIC_SHA_RESOLUTION_COMMAND} && "
    f'{RAW_PUBLIC_PREFLIGHT_COMMAND} && t="$(date +%s)" && '
    f"{CLONE_VERIFICATION_COMMAND}'"
)
OUTSIDE_RUN_ATTESTATION = (
    "I attest that I am not a Beater project maintainer, I received no "
    "step-by-step help beyond public repository instructions, I used a fresh "
    "clone, and I completed the Gate 2 flow unaided."
)
DIAGNOSTIC_ATTESTATION = (
    "Diagnostic maintainer full-run used a browser click to read the manual confirmation code; "
    "this is not outside-person evidence and cannot close Gate 2."
)
IMMUTABLE_LOG_URL = re.compile(
    r"https://github\.com/jadenfix/beater/actions/runs/[0-9]+(?:/job/[0-9]+)?"
)


def raw_public_preflight_command_for_sha(expected_commit):
    return (
        'preflight="$(mktemp "${TMPDIR:-/tmp}/beater-gate2-preflight.XXXXXX")" && '
        f'curl -fsSL "{RAW_PREFLIGHT_URL_PREFIX}/{expected_commit}/{RAW_PREFLIGHT_PATH}" '
        f'-o "$preflight" && BEATER_GATE2_EXPECTED_COMMIT="{expected_commit}" bash "$preflight"'
    )


def is_immutable_log_url(value):
    return bool(IMMUTABLE_LOG_URL.fullmatch(value))


UNRESOLVED_REQUIRED_VALUES = {
    "...",
    "\u2026",
    "unknown",
    "not requested",
    "not reported",
    "tbd",
    "todo",
}
EMBEDDED_PLACEHOLDER = re.compile(
    "(^|[\\s:;,/()_-])(\\.\\.\\.|\\u2026|tbd|todo)($|[\\s:;,/()_-])", re.I
)

LLM_OBSERVATION_FRAGMENTS = [
    "llm.call",
    "prompt",
    "completion",
    "model",
    "token breakdown",
    "cost",
    "latency",
    "confirmation code",
]
WATERFALL_OBSERVATION_FRAGMENTS = ["run", "turn", "step", "tool", "MCP"]

NEGATED_OBSERVATION = re.compile(
    r"\b(?:did\s+not|didn't|could\s+not|couldn't|cannot|can't|failed\s+to\s+see|"
    r"not\s+visible|not\s+shown|not\s+showing|missing|without)\b"
)
VISIBLE_OBSERVATION = re.compile(
    r"\b(?:saw|seen|visible|read|confirmed|verified|opened|clicked|showed|displayed|inspected)\b"
)


def clean_value(value):
    return value.strip().strip("`").strip()


def markdown_field_values(source_text, name):
    return [
        clean_value(match)
        for match in re.findall(
            r"^- " + re.escape(name) + r":[ \t]*(.*)$",
            source_text,
            re.MULTILINE,
        )
    ]


def contains_placeholder_fragment(value):
    return bool(EMBEDDED_PLACEHOLDER.search(value))


def is_unresolved_marker(value):
    cleaned = clean_value(value)
    return not cleaned or cleaned.lower() in UNRESOLVED_REQUIRED_VALUES


def is_unresolved_argument(value, *, allow_none=False):
    cleaned = clean_value(value)
    return (
        is_unresolved_marker(cleaned)
        or (cleaned.lower() == "none" and not allow_none)
        or contains_placeholder_fragment(cleaned)
    )


def observation_errors(field_name, value, required_fragments):
    normalized = value.lower()
    errors = []
    if NEGATED_OBSERVATION.search(normalized):
        errors.append(f"{field_name} must be a positive observation, not negated evidence")
    if not VISIBLE_OBSERVATION.search(normalized):
        errors.append(f"{field_name} must describe a positive visible observation")
    missing = [
        fragment for fragment in required_fragments if fragment.lower() not in normalized
    ]
    if missing:
        errors.append(f"{field_name} must mention: " + ", ".join(missing))
    return errors
