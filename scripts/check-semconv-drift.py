#!/usr/bin/env python3
"""Gate: every SDK's semconv + config must match the single source
(sdks/semconv/conventions.json, generated from beater-schema). Prevents the
hand-written SDK files from drifting from the server's span kinds, attribute
keys, defaults, and env var names.

This parses the VALUES actually ASSIGNED to constants (RHS of `=`/`:`), with
comments stripped, so a changed value can't hide behind a mention in a comment.
Exit 1 on any mismatch.
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
conv = json.loads((ROOT / "sdks/semconv/conventions.json").read_text())
required_strings = set(conv["span_kinds"]) | set(conv["attributes"].values())
required_config = set(conv["defaults"].values()) | set(conv["env"].values())
SEMCONV_WIRE_RE = re.compile(r"^(?:[a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+|x-beater-[a-z0-9-]+)$")

# (semconv file, config file, line-comment marker) per SDK.
SDKS = {
    "python": ("sdks/python/beater/semconv.py", "sdks/python/beater/config.py", "#"),
    "typescript": ("sdks/typescript/src/semconv.ts", "sdks/typescript/src/config.ts", "//"),
    "rust": ("sdks/rust/src/semconv.rs", "sdks/rust/src/config.rs", "//"),
    "go": ("sdks/go/semconv.go", "sdks/go/config.go", "//"),
    "java": (
        "sdks/java/src/main/java/ai/beater/sdk/SemConv.java",
        "sdks/java/src/main/java/ai/beater/sdk/BeaterConfig.java",
        "//",
    ),
}

def assigned_values(path: pathlib.Path, comment: str) -> set:
    """Double-quoted string literals appearing in CODE (not comments).

    Quote-aware: a line-comment marker inside a string (e.g. `//` in a URL) is not
    treated as a comment, and a canonical value mentioned only in a comment does
    not count. Matches any language idiom (assignment, method arg, ternary).
    """
    values = set()
    for line in path.read_text().splitlines():
        in_str, buf, i, n = False, [], 0, len(line)
        while i < n:
            ch = line[i]
            if in_str:
                if ch == "\\" and i + 1 < n:
                    buf.append(line[i + 1])
                    i += 2
                    continue
                if ch == '"':
                    values.add("".join(buf))
                    buf, in_str = [], False
                elif True:
                    buf.append(ch)
                i += 1
            else:
                if comment and line.startswith(comment, i):
                    break
                if ch == '"':
                    in_str = True
                i += 1
    return values

def semconv_wire_values(values: set) -> set:
    """Assigned strings that look like semantic-convention wire values."""
    return {value for value in values if SEMCONV_WIRE_RE.fullmatch(value)}


failed = False
for lang, (semconv_rel, config_rel, comment) in SDKS.items():
    sem_vals = semconv_wire_values(assigned_values(ROOT / semconv_rel, comment))
    cfg_vals = assigned_values(ROOT / config_rel, comment)
    missing_sem = sorted(required_strings - sem_vals)
    extra_sem = sorted(sem_vals - required_strings)
    missing_cfg = sorted(required_config - cfg_vals)
    if missing_sem or extra_sem or missing_cfg:
        failed = True
        print(f"FAIL {lang}:")
        if missing_sem:
            print(f"  semconv ({semconv_rel}) missing assigned values: {missing_sem}")
        if extra_sem:
            print(f"  semconv ({semconv_rel}) has extra assigned values: {extra_sem}")
        if missing_cfg:
            print(f"  config ({config_rel}) missing defaults/env values: {missing_cfg}")
    else:
        print(f"PASS {lang}: {len(sem_vals)} exact conventions + {len(required_config)} config values present")

if failed:
    print("\nSemconv drift detected. Update the SDK to match sdks/semconv/conventions.json "
          "(regenerate it with `cargo xtask regen-semconv`).", file=sys.stderr)
    sys.exit(1)
print("\nPASS: all SDK semconv + config match the single source.")
