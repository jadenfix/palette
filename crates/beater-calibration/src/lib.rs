use anyhow::{anyhow, Context};
use async_trait::async_trait;
use beater_core::{
    CalibrationReportId, DatasetCaseId, DatasetId, DatasetVersionId, EvaluatorVersionId, ProjectId,
    TenantId, Timestamp,
};
use beater_datasets::{DatasetEvalReport, DatasetVersionSnapshot};
use beater_schema::EvalResult;
use beater_store::{IntoStoreResult, StoreError, StoreResult};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationLabel {
    Pass,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CalibrationPolicy {
    pub pass_threshold: f64,
}

impl Default for CalibrationPolicy {
    fn default() -> Self {
        Self {
            pass_threshold: 0.5,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CalibrationConfusion {
    pub human_pass_judge_pass: usize,
    pub human_pass_judge_fail: usize,
    pub human_fail_judge_pass: usize,
    pub human_fail_judge_fail: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReliabilityBin {
    pub bin_index: usize,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub sample_count: usize,
    pub mean_confidence: Option<f64>,
    pub accuracy: Option<f64>,
    pub calibration_gap: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CalibrationItem {
    pub dataset_case_id: DatasetCaseId,
    pub human_label: CalibrationLabel,
    pub judge_label: CalibrationLabel,
    pub judge_score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge_result_label: Option<String>,
    pub agreed: bool,
    #[schema(value_type = serde_json::Value)]
    pub evidence: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CalibrationReport {
    pub calibration_report_id: CalibrationReportId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub dataset_id: DatasetId,
    pub dataset_version_id: DatasetVersionId,
    pub evaluator_version_id: EvaluatorVersionId,
    pub eval_report_id: String,
    pub policy: CalibrationPolicy,
    pub sample_count: usize,
    pub observed_agreement: f64,
    pub expected_agreement: f64,
    pub cohen_kappa: f64,
    pub brier_score: f64,
    pub expected_calibration_error: f64,
    pub reliability_bins: Vec<ReliabilityBin>,
    pub confusion: CalibrationConfusion,
    pub items: Vec<CalibrationItem>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: Timestamp,
}

const RELIABILITY_BIN_COUNT: usize = 10;

#[async_trait]
pub trait CalibrationStore: Send + Sync {
    async fn write_report(&self, report: CalibrationReport) -> StoreResult<CalibrationReport>;

    async fn get_report(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        calibration_report_id: CalibrationReportId,
    ) -> StoreResult<CalibrationReport>;

    async fn latest_report(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        dataset_id: DatasetId,
        dataset_version_id: DatasetVersionId,
        evaluator_version_id: Option<EvaluatorVersionId>,
    ) -> StoreResult<Option<CalibrationReport>>;
}

#[derive(Clone)]
pub struct SqliteCalibrationStore {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteCalibrationStore {
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection =
            Connection::open_in_memory().context("open in-memory calibration sqlite")?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create calibration sqlite dir {}", parent.display()))?;
        }
        let connection = Connection::open(path)
            .with_context(|| format!("open sqlite calibration store {}", path.display()))?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.init()?;
        Ok(store)
    }

    fn init(&self) -> anyhow::Result<()> {
        let connection = self.lock()?;
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS calibration_reports (
                    tenant_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    calibration_report_id TEXT NOT NULL,
                    dataset_id TEXT NOT NULL,
                    dataset_version_id TEXT NOT NULL,
                    evaluator_version_id TEXT NOT NULL,
                    eval_report_id TEXT NOT NULL,
                    cohen_kappa REAL NOT NULL,
                    observed_agreement REAL NOT NULL,
                    sample_count INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    report_json TEXT NOT NULL,
                    PRIMARY KEY (tenant_id, project_id, calibration_report_id)
                );

                CREATE INDEX IF NOT EXISTS idx_calibration_reports_latest
                  ON calibration_reports (
                    tenant_id, project_id, dataset_id, dataset_version_id,
                    evaluator_version_id, created_at DESC, calibration_report_id DESC
                  );
                "#,
            )
            .context("initialize sqlite calibration store")?;
        Ok(())
    }

    fn lock(&self) -> anyhow::Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|err| anyhow!("sqlite calibration connection mutex poisoned: {err}"))
    }
}

#[async_trait]
impl CalibrationStore for SqliteCalibrationStore {
    async fn write_report(&self, report: CalibrationReport) -> StoreResult<CalibrationReport> {
        let report_json = serde_json::to_string(&report)
            .context("serialize calibration report")
            .into_store()?;
        let connection = self.lock().into_store()?;
        connection
            .execute(
                r#"
                INSERT INTO calibration_reports
                  (tenant_id, project_id, calibration_report_id, dataset_id,
                   dataset_version_id, evaluator_version_id, eval_report_id, cohen_kappa,
                   observed_agreement, sample_count, created_at, report_json)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
                params![
                    report.tenant_id.as_str(),
                    report.project_id.as_str(),
                    report.calibration_report_id.as_str(),
                    report.dataset_id.as_str(),
                    report.dataset_version_id.as_str(),
                    report.evaluator_version_id.as_str(),
                    report.eval_report_id.as_str(),
                    report.cohen_kappa,
                    report.observed_agreement,
                    report.sample_count as i64,
                    report.created_at.to_rfc3339(),
                    report_json
                ],
            )
            .context("insert calibration report")
            .into_store()?;
        Ok(report)
    }

    async fn get_report(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        calibration_report_id: CalibrationReportId,
    ) -> StoreResult<CalibrationReport> {
        let connection = self.lock().into_store()?;
        let report_json = connection
            .query_row(
                r#"
                SELECT report_json
                FROM calibration_reports
                WHERE tenant_id = ?1 AND project_id = ?2 AND calibration_report_id = ?3
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    calibration_report_id.as_str()
                ],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query calibration report")
            .into_store()?
            .ok_or_else(|| {
                StoreError::NotFound(format!(
                    "calibration report {} not found",
                    calibration_report_id.as_str()
                ))
            })?;
        serde_json::from_str(&report_json)
            .context("decode calibration report")
            .into_store()
    }

    async fn latest_report(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        dataset_id: DatasetId,
        dataset_version_id: DatasetVersionId,
        evaluator_version_id: Option<EvaluatorVersionId>,
    ) -> StoreResult<Option<CalibrationReport>> {
        let evaluator_version_id = evaluator_version_id.as_ref().map(|id| id.as_str());
        let connection = self.lock().into_store()?;
        let report_json = connection
            .query_row(
                r#"
                SELECT report_json
                FROM calibration_reports
                WHERE tenant_id = ?1
                  AND project_id = ?2
                  AND dataset_id = ?3
                  AND dataset_version_id = ?4
                  AND (?5 IS NULL OR evaluator_version_id = ?5)
                ORDER BY created_at DESC, calibration_report_id DESC
                LIMIT 1
                "#,
                params![
                    tenant_id.as_str(),
                    project_id.as_str(),
                    dataset_id.as_str(),
                    dataset_version_id.as_str(),
                    evaluator_version_id
                ],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("query latest calibration report")
            .into_store()?;
        report_json
            .map(|report_json| {
                serde_json::from_str(&report_json).context("decode calibration report")
            })
            .transpose()
            .into_store()
    }
}

pub fn calibrate_eval_report(
    snapshot: &DatasetVersionSnapshot,
    eval_report: &DatasetEvalReport,
    policy: CalibrationPolicy,
) -> anyhow::Result<CalibrationReport> {
    ensure_report_matches_snapshot(snapshot, eval_report)?;
    let results = results_by_case(eval_report)?;
    let mut items = Vec::with_capacity(snapshot.cases.len());
    let mut confusion = CalibrationConfusion::default();

    for case in &snapshot.cases {
        let result = results
            .get(&case.case_id)
            .ok_or_else(|| anyhow!("eval report missing case {}", case.case_id.as_str()))?;
        let human_label = human_label_for_case(case)?;
        let judge_label = judge_label_for_result(result, policy.pass_threshold);
        let agreed = human_label == judge_label;
        add_confusion(&mut confusion, &human_label, &judge_label);
        items.push(CalibrationItem {
            dataset_case_id: case.case_id.clone(),
            human_label,
            judge_label,
            judge_score: result.score,
            judge_result_label: result.label.clone(),
            agreed,
            evidence: result.evidence.clone(),
        });
    }

    let sample_count = items.len();
    if sample_count == 0 {
        return Err(anyhow!("cannot calibrate an empty report"));
    }
    let observed_agreement =
        items.iter().filter(|item| item.agreed).count() as f64 / sample_count as f64;
    let expected_agreement = expected_agreement(&confusion, sample_count);
    let cohen_kappa = cohen_kappa(observed_agreement, expected_agreement);
    let brier_score = brier_score(&items)?;
    let reliability_bins = reliability_bins(&items, RELIABILITY_BIN_COUNT)?;
    let expected_calibration_error = expected_calibration_error(&reliability_bins, sample_count);

    Ok(CalibrationReport {
        calibration_report_id: CalibrationReportId::new(Uuid::new_v4().to_string())?,
        tenant_id: snapshot.tenant_id.clone(),
        project_id: snapshot.project_id.clone(),
        dataset_id: snapshot.dataset_id.clone(),
        dataset_version_id: snapshot.version_id.clone(),
        evaluator_version_id: eval_report.evaluator_version_id.clone(),
        eval_report_id: eval_report.report_id.clone(),
        policy,
        sample_count,
        observed_agreement,
        expected_agreement,
        cohen_kappa,
        brier_score,
        expected_calibration_error,
        reliability_bins,
        confusion,
        items,
        created_at: Utc::now(),
    })
}

fn ensure_report_matches_snapshot(
    snapshot: &DatasetVersionSnapshot,
    eval_report: &DatasetEvalReport,
) -> anyhow::Result<()> {
    if snapshot.tenant_id != eval_report.tenant_id
        || snapshot.project_id != eval_report.project_id
        || snapshot.dataset_id != eval_report.dataset_id
        || snapshot.version_id != eval_report.dataset_version_id
    {
        return Err(anyhow!(
            "eval report {} does not match dataset version {}",
            eval_report.report_id,
            snapshot.version_id.as_str()
        ));
    }
    Ok(())
}

fn results_by_case(
    eval_report: &DatasetEvalReport,
) -> anyhow::Result<BTreeMap<DatasetCaseId, &EvalResult>> {
    let mut results = BTreeMap::new();
    for result in &eval_report.results {
        let case_id = result.reproducibility.dataset_case_id.clone();
        if results.insert(case_id.clone(), result).is_some() {
            return Err(anyhow!(
                "eval report {} has duplicate result for case {}",
                eval_report.report_id,
                case_id.as_str()
            ));
        }
    }
    Ok(results)
}

fn human_label_for_case(case: &beater_datasets::DatasetCase) -> anyhow::Result<CalibrationLabel> {
    let reference = case.reference.as_ref().ok_or_else(|| {
        anyhow!(
            "dataset case {} has no human reference",
            case.case_id.as_str()
        )
    })?;
    Ok(if reference == &case.output {
        CalibrationLabel::Pass
    } else {
        CalibrationLabel::Fail
    })
}

fn judge_label_for_result(result: &EvalResult, pass_threshold: f64) -> CalibrationLabel {
    match result.label.as_deref() {
        Some("pass") => CalibrationLabel::Pass,
        Some("fail") => CalibrationLabel::Fail,
        _ if result.score >= pass_threshold => CalibrationLabel::Pass,
        _ => CalibrationLabel::Fail,
    }
}

fn add_confusion(
    confusion: &mut CalibrationConfusion,
    human: &CalibrationLabel,
    judge: &CalibrationLabel,
) {
    match (human, judge) {
        (CalibrationLabel::Pass, CalibrationLabel::Pass) => {
            confusion.human_pass_judge_pass += 1;
        }
        (CalibrationLabel::Pass, CalibrationLabel::Fail) => {
            confusion.human_pass_judge_fail += 1;
        }
        (CalibrationLabel::Fail, CalibrationLabel::Pass) => {
            confusion.human_fail_judge_pass += 1;
        }
        (CalibrationLabel::Fail, CalibrationLabel::Fail) => {
            confusion.human_fail_judge_fail += 1;
        }
    }
}

fn expected_agreement(confusion: &CalibrationConfusion, sample_count: usize) -> f64 {
    let n = sample_count as f64;
    let human_pass = (confusion.human_pass_judge_pass + confusion.human_pass_judge_fail) as f64 / n;
    let human_fail = (confusion.human_fail_judge_pass + confusion.human_fail_judge_fail) as f64 / n;
    let judge_pass = (confusion.human_pass_judge_pass + confusion.human_fail_judge_pass) as f64 / n;
    let judge_fail = (confusion.human_pass_judge_fail + confusion.human_fail_judge_fail) as f64 / n;
    human_pass * judge_pass + human_fail * judge_fail
}

fn cohen_kappa(observed_agreement: f64, expected_agreement: f64) -> f64 {
    let denominator = 1.0 - expected_agreement;
    if denominator.abs() < f64::EPSILON {
        if (observed_agreement - 1.0).abs() < f64::EPSILON {
            return 1.0;
        }
        return 0.0;
    }
    (observed_agreement - expected_agreement) / denominator
}

pub fn brier_score(items: &[CalibrationItem]) -> anyhow::Result<f64> {
    if items.is_empty() {
        return Err(anyhow!("cannot compute Brier score for an empty sample"));
    }
    let mut total = 0.0;
    for item in items {
        let probability = probability(item)?;
        let outcome = outcome(item);
        total += (probability - outcome).powi(2);
    }
    Ok(total / items.len() as f64)
}

pub fn reliability_bins(
    items: &[CalibrationItem],
    bin_count: usize,
) -> anyhow::Result<Vec<ReliabilityBin>> {
    if items.is_empty() {
        return Err(anyhow!(
            "cannot compute reliability bins for an empty sample"
        ));
    }
    if bin_count == 0 {
        return Err(anyhow!("reliability bin count must be greater than zero"));
    }

    let width = 1.0 / bin_count as f64;
    let mut sums = vec![0.0; bin_count];
    let mut correct = vec![0.0; bin_count];
    let mut counts = vec![0usize; bin_count];

    for item in items {
        let probability = probability(item)?;
        let index = probability_bin(probability, bin_count);
        sums[index] += probability;
        correct[index] += outcome(item);
        counts[index] += 1;
    }

    Ok((0..bin_count)
        .map(|index| {
            let lower_bound = index as f64 * width;
            let upper_bound = if index + 1 == bin_count {
                1.0
            } else {
                (index + 1) as f64 * width
            };
            let sample_count = counts[index];
            if sample_count == 0 {
                return ReliabilityBin {
                    bin_index: index,
                    lower_bound,
                    upper_bound,
                    sample_count,
                    mean_confidence: None,
                    accuracy: None,
                    calibration_gap: None,
                };
            }
            let mean_confidence = sums[index] / sample_count as f64;
            let accuracy = correct[index] / sample_count as f64;
            ReliabilityBin {
                bin_index: index,
                lower_bound,
                upper_bound,
                sample_count,
                mean_confidence: Some(mean_confidence),
                accuracy: Some(accuracy),
                calibration_gap: Some((accuracy - mean_confidence).abs()),
            }
        })
        .collect())
}

pub fn expected_calibration_error(bins: &[ReliabilityBin], sample_count: usize) -> f64 {
    if sample_count == 0 {
        return 0.0;
    }
    bins.iter()
        .filter_map(|bin| {
            bin.calibration_gap
                .map(|gap| bin.sample_count as f64 / sample_count as f64 * gap)
        })
        .sum()
}

fn probability(item: &CalibrationItem) -> anyhow::Result<f64> {
    if !item.judge_score.is_finite() {
        return Err(anyhow!(
            "calibration score for case {} is not finite",
            item.dataset_case_id.as_str()
        ));
    }
    if !(0.0..=1.0).contains(&item.judge_score) {
        return Err(anyhow!(
            "calibration score for case {} must be between 0 and 1, got {}",
            item.dataset_case_id.as_str(),
            item.judge_score
        ));
    }
    Ok(item.judge_score)
}

fn outcome(item: &CalibrationItem) -> f64 {
    match item.human_label {
        CalibrationLabel::Pass => 1.0,
        CalibrationLabel::Fail => 0.0,
    }
}

fn probability_bin(probability: f64, bin_count: usize) -> usize {
    if probability >= 1.0 {
        return bin_count - 1;
    }
    (probability * bin_count as f64).floor() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_core::{AgentReleaseId, EnvironmentId, EvalResultId, SpanId, TraceId};
    use beater_datasets::DatasetCase;
    use beater_schema::EvalReproducibility;
    use serde_json::json;

    #[tokio::test]
    async fn calibration_computes_kappa_and_persists_report() -> anyhow::Result<()> {
        let tenant = TenantId::new("tenant")?;
        let project = ProjectId::new("project")?;
        let dataset = DatasetId::new("dataset")?;
        let version = DatasetVersionId::new("version")?;
        let evaluator = EvaluatorVersionId::new("judge-v1")?;
        let snapshot = DatasetVersionSnapshot {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            version_id: version.clone(),
            cases: vec![
                fixture_case(
                    &tenant,
                    &project,
                    &dataset,
                    "case-1",
                    json!("a"),
                    json!("a"),
                )?,
                fixture_case(
                    &tenant,
                    &project,
                    &dataset,
                    "case-2",
                    json!("b"),
                    json!("c"),
                )?,
                fixture_case(
                    &tenant,
                    &project,
                    &dataset,
                    "case-3",
                    json!("d"),
                    json!("d"),
                )?,
                fixture_case(
                    &tenant,
                    &project,
                    &dataset,
                    "case-4",
                    json!("e"),
                    json!("f"),
                )?,
            ],
            created_at: Utc::now(),
        };
        let eval_report = DatasetEvalReport {
            report_id: "eval-report".to_string(),
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            dataset_version_id: version.clone(),
            evaluator_version_id: evaluator.clone(),
            result_count: 4,
            aggregate_score: 0.5,
            results: vec![
                fixture_result(&snapshot, "case-1", evaluator.clone(), 1.0, "pass")?,
                fixture_result(&snapshot, "case-2", evaluator.clone(), 1.0, "pass")?,
                fixture_result(&snapshot, "case-3", evaluator.clone(), 0.0, "fail")?,
                fixture_result(&snapshot, "case-4", evaluator.clone(), 0.0, "fail")?,
            ],
            created_at: Utc::now(),
        };

        let report = calibrate_eval_report(&snapshot, &eval_report, CalibrationPolicy::default())?;

        assert_eq!(report.sample_count, 4);
        assert_eq!(report.observed_agreement, 0.5);
        assert_eq!(report.expected_agreement, 0.5);
        assert_eq!(report.cohen_kappa, 0.0);
        assert_eq!(report.confusion.human_pass_judge_pass, 1);
        assert_eq!(report.confusion.human_pass_judge_fail, 1);
        assert_eq!(report.confusion.human_fail_judge_pass, 1);
        assert_eq!(report.confusion.human_fail_judge_fail, 1);
        assert_eq!(report.brier_score, 0.5);
        assert_eq!(report.expected_calibration_error, 0.5);
        assert_eq!(report.reliability_bins.len(), RELIABILITY_BIN_COUNT);
        assert_eq!(report.reliability_bins[0].sample_count, 2);
        assert_eq!(report.reliability_bins[0].mean_confidence, Some(0.0));
        assert_eq!(report.reliability_bins[0].accuracy, Some(0.5));
        assert_eq!(report.reliability_bins[0].calibration_gap, Some(0.5));
        assert_eq!(report.reliability_bins[9].sample_count, 2);
        assert_eq!(report.reliability_bins[9].mean_confidence, Some(1.0));
        assert_eq!(report.reliability_bins[9].accuracy, Some(0.5));
        assert_eq!(report.reliability_bins[9].calibration_gap, Some(0.5));

        let store = SqliteCalibrationStore::in_memory()?;
        let stored = store.write_report(report.clone()).await?;
        let loaded = store
            .get_report(
                tenant.clone(),
                project.clone(),
                stored.calibration_report_id.clone(),
            )
            .await?;
        assert_eq!(loaded.confusion, report.confusion);
        let latest = store
            .latest_report(tenant, project, dataset, version, Some(evaluator))
            .await?
            .ok_or_else(|| anyhow!("missing latest report"))?;
        assert_eq!(latest.calibration_report_id, stored.calibration_report_id);
        Ok(())
    }

    #[test]
    fn proper_scoring_metrics_handle_calibrated_probabilities() -> anyhow::Result<()> {
        let items = vec![
            calibration_item("case-1", CalibrationLabel::Fail, 0.0)?,
            calibration_item("case-2", CalibrationLabel::Pass, 1.0)?,
            calibration_item("case-3", CalibrationLabel::Fail, 0.25)?,
            calibration_item("case-4", CalibrationLabel::Pass, 0.75)?,
        ];

        let brier = brier_score(&items)?;
        assert!((brier - 0.03125).abs() < 1e-12);

        let bins = reliability_bins(&items, 4)?;
        assert_eq!(bins.len(), 4);
        assert_eq!(bins[0].sample_count, 1);
        assert_eq!(bins[0].mean_confidence, Some(0.0));
        assert_eq!(bins[0].accuracy, Some(0.0));
        assert_eq!(bins[0].calibration_gap, Some(0.0));
        assert_eq!(bins[1].sample_count, 1);
        assert_eq!(bins[1].mean_confidence, Some(0.25));
        assert_eq!(bins[1].accuracy, Some(0.0));
        assert_eq!(bins[2].sample_count, 0);
        assert_eq!(bins[2].mean_confidence, None);
        assert_eq!(bins[3].sample_count, 2);
        assert_eq!(bins[3].mean_confidence, Some(0.875));
        assert_eq!(bins[3].accuracy, Some(1.0));

        let ece = expected_calibration_error(&bins, items.len());
        assert!((ece - 0.125).abs() < 1e-12);
        Ok(())
    }

    #[test]
    fn proper_scoring_rejects_invalid_probability_scores() -> anyhow::Result<()> {
        let items = vec![calibration_item("case-1", CalibrationLabel::Pass, 1.2)?];

        let Err(error) = brier_score(&items) else {
            panic!("expected brier_score to reject out-of-range probability");
        };
        assert!(error.to_string().contains("must be between 0 and 1"));
        Ok(())
    }

    fn calibration_item(
        case_id: &str,
        human_label: CalibrationLabel,
        judge_score: f64,
    ) -> anyhow::Result<CalibrationItem> {
        let judge_label = if judge_score >= 0.5 {
            CalibrationLabel::Pass
        } else {
            CalibrationLabel::Fail
        };
        Ok(CalibrationItem {
            dataset_case_id: DatasetCaseId::new(case_id)?,
            agreed: human_label == judge_label,
            human_label,
            judge_label,
            judge_score,
            judge_result_label: None,
            evidence: json!({}),
        })
    }

    fn fixture_case(
        tenant: &TenantId,
        project: &ProjectId,
        dataset: &DatasetId,
        case_id: &str,
        output: Value,
        reference: Value,
    ) -> anyhow::Result<DatasetCase> {
        Ok(DatasetCase {
            tenant_id: tenant.clone(),
            project_id: project.clone(),
            dataset_id: dataset.clone(),
            case_id: DatasetCaseId::new(case_id)?,
            source_trace_id: TraceId::new(format!("trace-{case_id}"))?,
            source_span_id: SpanId::new(format!("span-{case_id}"))?,
            source_environment_id: EnvironmentId::new("prod")?,
            input: json!("question"),
            output,
            reference: Some(reference),
            trace: json!({}),
            normalizer_version: "test".to_string(),
            trace_schema_version: 1,
            input_artifact_hashes: Vec::new(),
            created_at: Utc::now(),
        })
    }

    fn fixture_result(
        snapshot: &DatasetVersionSnapshot,
        case_id: &str,
        evaluator_version_id: EvaluatorVersionId,
        score: f64,
        label: &str,
    ) -> anyhow::Result<EvalResult> {
        Ok(EvalResult {
            eval_result_id: EvalResultId::new(Uuid::new_v4().to_string())?,
            tenant_id: snapshot.tenant_id.clone(),
            project_id: snapshot.project_id.clone(),
            trace_id: TraceId::new(format!("trace-{case_id}"))?,
            span_id: Some(SpanId::new(format!("span-{case_id}"))?),
            score,
            label: Some(label.to_string()),
            evidence: json!({ "label": label }),
            reproducibility: EvalReproducibility {
                dataset_version_id: snapshot.version_id.clone(),
                dataset_case_id: DatasetCaseId::new(case_id)?,
                agent_release_id: AgentReleaseId::new("agent")?,
                prompt_version_id: None,
                evaluator_version_id,
                code_hash: None,
                wasm_hash: None,
                wasi_abi_version: None,
                judge_model_id: Some("judge-model".to_string()),
                judge_provider: Some("openai".to_string()),
                judge_parameters: json!({}),
                judge_seed: None,
                judge_rubric_version: Some("judge-v1".to_string()),
                normalizer_version: "test".to_string(),
                trace_schema_version: 1,
                input_artifact_hashes: Vec::new(),
            },
            cost: None,
            tokens: None,
            created_at: Utc::now(),
            non_reproducible_reason: None,
        })
    }
}
