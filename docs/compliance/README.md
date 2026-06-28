# Compliance Evidence Index

This directory is the planned home for Beater compliance evidence. It is not a
SOC 2, HIPAA, GDPR, or enterprise-readiness claim.

`ARCHITECTURE.md` §20.7 #5.11 names the expected compliance surface:

- SOC 2 control matrix
- access-review runbook
- incident-response plan
- subprocessor list
- DPA template

Current source-of-truth documents:

- [`SECURITY.md`](../../SECURITY.md) covers coordinated disclosure and
  self-host hardening.
- [`GOVERNANCE.md`](../../GOVERNANCE.md) covers project governance and maintainer
  authority.
- [`CONTRIBUTING.md`](../../CONTRIBUTING.md) covers contribution and CI gates.
- [`docs/feature-matrix.md`](../feature-matrix.md) distinguishes OSS and hosted
  scope.
- [`docs/architecture-status.md`](../architecture-status.md) tracks built,
  partial, planned, and deferred architecture items.

Planned evidence should stay status-ledger style: each page must say what is
built, what is planned, and which verification command or audit artifact backs
the claim. Do not use this directory to imply compliance certification before
the controls, review cadence, and restore/security drills are implemented and
verified.
