use beater_audit::SqliteAuditStore;
use beater_auth::SqliteApiKeyStore;
use beater_bus::SqliteDurableBus;
use beater_calibration::SqliteCalibrationStore;
use beater_datasets::SqliteDatasetStore;
use beater_experiments::SqliteExperimentStore;
use beater_gates::SqliteGateStore;
use beater_human::SqliteHumanReviewStore;
use beater_judge::SqliteJudgeLedger;
use beater_replay::SqliteReplayStore;
use beater_secrets::{EncryptedSqliteProviderSecretStore, SecretKeyring};
use beater_store_sql::{
    SqliteMetadataStore, SqliteQuotaLimiter, SqliteTraceStore, migrate_local_beaterd_sqlite,
};
use beater_usage::SqliteUsageLedger;

#[test]
fn local_sqlite_migration_bootstraps_runtime_store_schemas() -> anyhow::Result<()> {
    let tempdir = tempfile::tempdir()?;
    let path = tempdir.path().join("beater.sqlite");

    let report = migrate_local_beaterd_sqlite(&path)?;
    assert_eq!(report.applied, 1);
    assert_eq!(report.skipped, 0);

    let _traces = SqliteTraceStore::open(&path)?;
    let _metadata = SqliteMetadataStore::open(&path)?;
    let _quota = SqliteQuotaLimiter::open(&path)?;
    let _bus = SqliteDurableBus::open(&path, 16)?;
    let _api_keys = SqliteApiKeyStore::open(&path)?;
    let _provider_secrets =
        EncryptedSqliteProviderSecretStore::open(&path, SecretKeyring::generated_for_tests()?)?;
    let _datasets = SqliteDatasetStore::open(&path)?;
    let _experiments = SqliteExperimentStore::open(&path)?;
    let _gates = SqliteGateStore::open(&path)?;
    let _reviews = SqliteHumanReviewStore::open(&path)?;
    let _judge = SqliteJudgeLedger::open(&path)?;
    let _usage = SqliteUsageLedger::open(&path)?;
    let _audit = SqliteAuditStore::open(&path)?;
    let _replay = SqliteReplayStore::open(&path)?;
    let _calibration = SqliteCalibrationStore::open(&path)?;

    let second_report = migrate_local_beaterd_sqlite(&path)?;
    assert_eq!(second_report.applied, 0);
    assert_eq!(second_report.skipped, 1);

    Ok(())
}
