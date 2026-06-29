#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import textwrap
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "check-openapi-drift.sh"

GENERATED_OPENAPI = '{"openapi":"3.1.0","info":{"title":"Generated"}}\n'
GENERATED_TYPES = "export interface paths {}\n"


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def write_executable(path: Path, text: str) -> None:
    write(path, text)
    path.chmod(0o755)


def install_stubs(bin_dir: Path, log_path: Path) -> None:
    write_executable(
        bin_dir / "cargo",
        textwrap.dedent(
            f"""\
            #!/usr/bin/env python3
            import json
            import os
            import sys
            from pathlib import Path

            log_path = Path({str(log_path)!r})
            args = sys.argv[1:]
            with log_path.open("a") as log:
                log.write(json.dumps({{"cmd": "cargo", "cwd": os.getcwd(), "args": args}}) + "\\n")

            expected = ["run", "-q", "-p", "beater-api", "--example", "dump_openapi"]
            if args != expected:
                sys.stderr.write(f"unexpected cargo args: {{args!r}}\\n")
                sys.exit(2)

            sys.stdout.write({GENERATED_OPENAPI!r})
            """
        ),
    )
    write_executable(
        bin_dir / "npx",
        textwrap.dedent(
            f"""\
            #!/usr/bin/env python3
            import json
            import os
            import sys
            from pathlib import Path

            log_path = Path({str(log_path)!r})
            args = sys.argv[1:]
            if len(args) != 4 or args[0] != "openapi-typescript" or args[2] != "-o":
                sys.stderr.write(f"unexpected npx args: {{args!r}}\\n")
                sys.exit(2)

            source = Path(args[1])
            output = Path(args[3])
            input_text = source.read_text()
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text({GENERATED_TYPES!r})

            with log_path.open("a") as log:
                log.write(
                    json.dumps(
                        {{
                            "cmd": "npx",
                            "cwd": os.getcwd(),
                            "args": args,
                            "input_text": input_text,
                        }}
                    )
                    + "\\n"
                )
            """
        ),
    )
    write_executable(
        bin_dir / "diff",
        textwrap.dedent(
            """\
            #!/usr/bin/env python3
            import json
            import os
            import sys
            from pathlib import Path

            log_path = Path(os.environ["OPENAPI_DRIFT_TEST_LOG"])
            args = sys.argv[1:]
            if len(args) != 3 or args[0] != "-u":
                sys.stderr.write(f"unexpected diff args: {args!r}\\n")
                sys.exit(2)

            left = Path(args[1])
            right = Path(args[2])
            left_text = left.read_text()
            right_text = right.read_text()
            with log_path.open("a") as log:
                log.write(
                    json.dumps(
                        {
                            "cmd": "diff",
                            "cwd": os.getcwd(),
                            "args": args,
                            "left_text": left_text,
                            "right_text": right_text,
                        }
                    )
                    + "\\n"
                )

            if left_text != right_text:
                sys.stderr.write(f"DRIFT: {left} differs from {right}\\n")
                sys.exit(1)
            """
        ),
    )


def prepare_repo(
    root: Path,
    *,
    openapi_snapshot: str = GENERATED_OPENAPI,
    types_snapshot: str = GENERATED_TYPES,
) -> tuple[Path, Path, Path]:
    script = root / "scripts" / "check-openapi-drift.sh"
    script.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(SCRIPT, script)
    script.chmod(0o755)

    write(root / "web/dashboard/openapi/beater-read-api.json", openapi_snapshot)
    write(root / "web/dashboard/lib/generated/api-types.ts", types_snapshot)

    bin_dir = root / "fake-bin"
    log_path = root / "command-log.jsonl"
    bin_dir.mkdir()
    install_stubs(bin_dir, log_path)
    return script, bin_dir, log_path


def run_script(script: Path, root: Path, bin_dir: Path, log_path: Path) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["PATH"] = f"{bin_dir}{os.pathsep}{env['PATH']}"
    env["OPENAPI_DRIFT_TEST_LOG"] = str(log_path)
    return subprocess.run(
        [str(script)],
        cwd=root,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )


def read_log(log_path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in log_path.read_text().splitlines()]


def test_generates_temp_openapi_runs_dashboard_codegen_and_diffs_artifacts() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp).resolve()
        script, bin_dir, log_path = prepare_repo(root)

        result = run_script(script, root, bin_dir, log_path)

        assert result.returncode == 0, result.stderr
        assert "OpenAPI snapshot and generated dashboard client are current." in result.stdout

        records = read_log(log_path)
        assert [record["cmd"] for record in records] == ["cargo", "npx", "diff", "diff"]

        cargo = records[0]
        assert cargo["cwd"] == str(root)
        assert cargo["args"] == ["run", "-q", "-p", "beater-api", "--example", "dump_openapi"]

        npx = records[1]
        npx_args = npx["args"]
        assert npx["cwd"] == str(root / "web/dashboard")
        assert npx_args[0] == "openapi-typescript"
        assert Path(npx_args[1]).name == "beater-read-api.json"
        assert npx_args[1] != str(root / "web/dashboard/openapi/beater-read-api.json")
        assert npx_args[2] == "-o"
        assert Path(npx_args[3]).name == "api-types.ts"
        assert npx["input_text"] == GENERATED_OPENAPI

        openapi_diff = records[2]
        assert openapi_diff["cwd"] == str(root)
        assert openapi_diff["args"][0] == "-u"
        assert openapi_diff["args"][1] == str(root / "web/dashboard/openapi/beater-read-api.json")
        assert Path(openapi_diff["args"][2]).name == "beater-read-api.json"
        assert openapi_diff["left_text"] == GENERATED_OPENAPI
        assert openapi_diff["right_text"] == GENERATED_OPENAPI

        types_diff = records[3]
        assert types_diff["cwd"] == str(root)
        assert types_diff["args"][0] == "-u"
        assert types_diff["args"][1] == str(root / "web/dashboard/lib/generated/api-types.ts")
        assert Path(types_diff["args"][2]).name == "api-types.ts"
        assert types_diff["left_text"] == GENERATED_TYPES
        assert types_diff["right_text"] == GENERATED_TYPES


def test_fails_when_openapi_snapshot_diff_reports_drift() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp).resolve()
        script, bin_dir, log_path = prepare_repo(root, openapi_snapshot='{"openapi":"stale"}\n')

        result = run_script(script, root, bin_dir, log_path)

        assert result.returncode == 1
        assert "DRIFT:" in result.stderr
        assert "OpenAPI snapshot and generated dashboard client are current." not in result.stdout

        records = read_log(log_path)
        assert [record["cmd"] for record in records] == ["cargo", "npx", "diff"]
        assert records[-1]["args"][1] == str(root / "web/dashboard/openapi/beater-read-api.json")


def test_fails_when_generated_dashboard_client_diff_reports_drift() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp).resolve()
        script, bin_dir, log_path = prepare_repo(root, types_snapshot="export interface paths { stale: true }\n")

        result = run_script(script, root, bin_dir, log_path)

        assert result.returncode == 1
        assert "DRIFT:" in result.stderr
        assert "OpenAPI snapshot and generated dashboard client are current." not in result.stdout

        records = read_log(log_path)
        assert [record["cmd"] for record in records] == ["cargo", "npx", "diff", "diff"]
        assert records[-1]["args"][1] == str(root / "web/dashboard/lib/generated/api-types.ts")


def main() -> None:
    test_generates_temp_openapi_runs_dashboard_codegen_and_diffs_artifacts()
    test_fails_when_openapi_snapshot_diff_reports_drift()
    test_fails_when_generated_dashboard_client_diff_reports_drift()
    print("check-openapi-drift tests passed.")


if __name__ == "__main__":
    main()
