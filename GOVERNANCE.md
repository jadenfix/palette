# Beater governance and the no-rug-pull promise

This document states how Beater is governed and, most importantly, the
durability commitments we make to people who self-host or build on Beater. It
satisfies requirement **R12.2** (no-rug-pull promise is documented) and pairs
with the root [`LICENSE`](LICENSE) and [`docs/feature-matrix.md`](docs/feature-matrix.md)
(the open-core boundary, R12.1).

## The no-rug-pull promise

We have all watched open-source infrastructure get relicensed out from under the
people who depend on it. Beater commits, in public and before launch, that this
will not happen here:

1. **The license will not be revoked.** The `beaterd` server, all 11 SDK clients,
   the MCP tools, the CLI, the canonical schema, and the dashboard are licensed
   under **Apache-2.0** and will remain under an OSI-approved permissive license.
   We will not move the open core to BSL, SSPL, Elastic License, or any
   "source-available" non-open license.

2. **No retroactive relicensing.** Any code released under Apache-2.0 stays
   under Apache-2.0 for that release. A future commercial decision cannot
   un-publish or relicense a version you already have. You can always fork the
   last permissively licensed commit.

3. **The contract stays open.** The OpenAPI `/v1` contract
   (`sdks/openapi/beater-api.json`), the semantic conventions
   (`crates/beater-schema` / `sdks/semconv`), and the data model are part of the
   open core. We will not paywall the protocol your agents emit to or the schema
   your traces are stored in. See `docs/sdk-platform-architecture.md` for the
   stability/deprecation policy of `/v1`.

4. **Self-host stays first-class and standalone.** OSS runs without Beater Cloud
   (R1.3). The all-in-one `beaterd` is the only mandatory deployment (R1.2), and
   self-host telemetry is opt-out and off by default (R12.5). We will not
   introduce a mandatory phone-home, a license-key check, or a "community edition
   that can't actually run in production" gate.

5. **Open-core boundary changes are additive and announced.** New commercial
   features are layered *on top of* the open contract (hosting, scale, support).
   We will not remove a capability from the open-source core to move it behind a
   paywall. If the boundary in `docs/feature-matrix.md` ever changes, it changes
   forward (more open, or new paid add-ons) and is announced in the changelog.

## Decision-making

- **Technical direction** is decided in the open via GitHub issues and pull
  requests. The contract-is-the-single-source-of-truth rule (`CONTRIBUTING.md`)
  is enforced by CI for everyone, including maintainers.
- **Breaking `/v1` changes** are blocked by the contract gates
  (`scripts/check-contract-sync.sh`, `oasdiff`) and follow the deprecation policy
  in `docs/sdk-platform-architecture.md`.
- **Maintainership** is open to contributors who demonstrate sustained,
  high-quality contributions; see `CONTRIBUTING.md` for the contribution path.

## Trademark

The Apache-2.0 license covers the code, not the "Beater" name and logo. You may
fork and redistribute the code; please do not imply official endorsement.

## Questions

Open a GitHub issue using the templates in `.github/ISSUE_TEMPLATE/`. Governance
questions are welcome.
