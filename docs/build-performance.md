# Build performance

Why a clean `cargo build`/`cargo test` over the whole workspace is slow, what
this repo does about it, and what you can opt into locally (especially on macOS).

## The floor is real

A few dependencies are inherently large compile units and are genuinely needed:

| Crate | Heavy dependency | Pulls in |
| --- | --- | --- |
| `beater-sandbox` | `wasmtime` + `cranelift` | the WASI runtime + JIT codegen |
| `beater-archive` | `datafusion` 54, `arrow`, `parquet` | the whole query/columnar stack |
| `beater-search` | `tantivy` | full-text index engine |
| `beater-browser-cdp` | `chromiumoxide` | CDP client + async-tungstenite |
| `beater-otlp`, bins | `tonic` + `prost` | gRPC + protobuf codegen |

A full `cargo build --workspace` compiles all of them, so first/clean builds are
slow no matter what. That part is "just how it goes."

## The good news: heavy deps are isolated

Each heavy dependency lives behind exactly one crate (table above). So you do
**not** pay for them unless you build a crate that needs them:

```bash
cargo build -p beater-core        # no datafusion / wasmtime / tantivy
cargo test  -p beater-eval        # no browser / archive stack
./beater-cli test backend         # full workspace — pays for everything
```

When iterating, build/test the single crate you're working on. Reach for the
whole workspace only when you need the cross-crate check (or before pushing).

## What's committed for everyone

`[profile.dev]` / `[profile.test]` in the root `Cargo.toml` set
`debug = "line-tables-only"`. Full debuginfo is the biggest avoidable slice of
dev build + link time (the macOS linker is especially sensitive to it).
`line-tables-only` keeps panic/backtrace `file:line` info — `RUST_BACKTRACE`
still resolves — while dropping the variable-level debuginfo a normal
edit→build→test loop never reads.

If you need to step through code in a debugger and inspect locals, temporarily
set `debug = true` (don't commit that).

Incremental compilation is on by default for dev/test; nothing to configure.

## Opt-in local wins (not committed — environment-specific)

These depend on tools you install, so they live in your **personal**
`~/.cargo/config.toml` (or an untracked `.cargo/config.toml`), not in the repo.

### Faster linker

Linking dominates incremental rebuilds. Pick the fast linker for your platform:

```toml
# macOS — use the LLVM linker (brew install llvm) or the newer system linker.
[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]   # needs: brew install lld
# Apple Silicon with recent Xcode also has a fast default linker; measure both.

# Linux
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]   # needs: apt-get install lld
# or mold (apt-get install mold): link-arg=-fuse-ld=mold
```

### sccache (cache compiled crates across builds/branches)

```bash
cargo install sccache         # or: brew install sccache
export RUSTC_WRAPPER=sccache  # in your shell profile
```

Most useful when you switch branches a lot — the heavy deps above get reused
from cache instead of recompiled.

## Container builds

The `Dockerfile` uses [`cargo-chef`](https://github.com/LukeMathWalker/cargo-chef):
dependencies are cooked once in a layer keyed on `recipe.json`, so a source-only
change reuses the cached dependency build. `beaterd` and `beaterctl` build in
separate `rust-deps`-derived stages, which lets BuildKit run them in parallel and
cache each binary independently.

## Quick reference

```bash
./beater-cli test backend     # mirrors CI's backend bucket locally
cargo build -p <crate>        # iterate on one crate (skips unrelated heavy deps)
cargo test  -p <crate>
```
