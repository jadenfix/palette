#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "regen-sdks.sh"
LANGS = ("rust", "python", "typescript", "go", "java", "c", "cpp")
SPEC = '{"openapi":"3.1.0"}\n'


def write_executable(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body)
    path.chmod(0o755)


def seed_temp_repo(repo: Path, generated: str) -> Path:
    script = repo / "scripts" / "regen-sdks.sh"
    script.parent.mkdir(parents=True)
    shutil.copy2(SCRIPT, script)
    script.chmod(0o755)

    spec = repo / "sdks" / "openapi" / "beater-api.json"
    dashboard = repo / "web" / "dashboard" / "openapi" / "beater-read-api.json"
    spec.parent.mkdir(parents=True)
    dashboard.parent.mkdir(parents=True)
    spec.write_text(SPEC)
    dashboard.write_text(SPEC)

    for lang in LANGS:
        out = repo / "sdks" / "clients" / lang
        out.mkdir(parents=True)
        (out / "generated.txt").write_text(generated)
    return script


def seed_fake_bin(bin_dir: Path) -> None:
    write_executable(
        bin_dir / "cargo",
        f"""#!/usr/bin/env bash
if [ "$*" = "run -q -p beater-api --example dump_openapi" ]; then
  printf '{SPEC}'
  exit 0
fi
exit 1
""",
    )
    write_executable(
        bin_dir / "docker",
        """#!/usr/bin/env bash
case "${1:-}" in
  pull)
    exit 0
    ;;
  run)
    out=""
    while [ "$#" -gt 0 ]; do
      if [ "$1" = "-o" ]; then
        shift
        out="$1"
      fi
      shift || true
    done
    out="${out#/local/}"
    mkdir -p "$PWD/$out"
    printf '%s' "${BEATER_TEST_GENERATED:-generated-v1}" > "$PWD/$out/generated.txt"
    exit 0
    ;;
esac
exit 1
""",
    )


def run_regen_check(
    before: str,
    generated: str,
) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory() as temp:
        temp_dir = Path(temp)
        script = seed_temp_repo(temp_dir / "repo", before)
        bin_dir = temp_dir / "bin"
        seed_fake_bin(bin_dir)

        env = os.environ.copy()
        env["PATH"] = f"{bin_dir}{os.pathsep}{env['PATH']}"
        env["BEATER_TEST_GENERATED"] = generated

        return subprocess.run(
            ["bash", str(script), "--check"],
            cwd=temp_dir / "repo",
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )


def test_regen_check_passes_when_dirty_generated_tree_is_current() -> None:
    result = run_regen_check("generated-v1", "generated-v1")

    assert result.returncode == 0, result.stderr
    assert "No drift: spec and all SDK clients are current." in result.stdout


def test_regen_check_fails_when_regeneration_changes_generated_tree() -> None:
    result = run_regen_check("generated-v1", "generated-v2")

    assert result.returncode == 1
    assert "ERROR: generated artifacts are stale" in result.stderr
    assert "sdks/clients" in result.stderr


def main() -> None:
    for test in (
        test_regen_check_passes_when_dirty_generated_tree_is_current,
        test_regen_check_fails_when_regeneration_changes_generated_tree,
    ):
        test()
    print("regen-sdks tests passed")


if __name__ == "__main__":
    main()
