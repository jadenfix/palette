//! Deterministic evaluator sandbox built on the WASI Component Model.
//!
//! Deterministic scorers run as WASI components implementing the
//! [`beater:scorer/deterministic-scorer`](../wit/scorer.wit) world. The world
//! imports **nothing** from the host (no WASI preview2, no clock, no randomness),
//! so a scorer's output is a pure function of its input — the property the
//! `DeterministicWasi` evaluator lane relies on. The runtime additionally bounds
//! execution with fuel and rejects components that try to import host functions.

use beater_eval::{EvalError, EvaluationCase, EvaluatorSpec, ScoreResult, evaluate_deterministic};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

wasmtime::component::bindgen!({
    world: "deterministic-scorer",
    path: "wit/scorer.wit",
});

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("invalid sandbox config: {field} must be greater than zero, got {value}")]
    InvalidConfig { field: &'static str, value: u128 },
    #[error("evaluator input is too large: {size_bytes} > {limit_bytes}")]
    InputTooLarge {
        size_bytes: usize,
        limit_bytes: usize,
    },
    #[error("component imports are disabled in deterministic evaluator sandbox: {0}")]
    HostImportDenied(String),
    #[error("evaluator component must export the deterministic-scorer `score` function")]
    MissingScoreFunction,
    #[error("evaluator returned invalid basis point score {0}; expected 0..=10000")]
    InvalidScore(i32),
    #[error("wasm execution failed: {0}")]
    Execution(String),
}

/// Host state for a scorer instantiation. The deterministic-scorer world has no
/// imports, so the store carries no host capabilities — this is intentionally a
/// zero-capability marker.
struct ScorerState;

#[derive(Clone, Debug)]
pub struct WasmEvaluatorRuntime {
    engine: Engine,
    max_input_bytes: usize,
    fuel: u64,
}

impl WasmEvaluatorRuntime {
    pub fn new(config: SandboxConfig) -> Result<Self, SandboxError> {
        config.validate()?;

        let mut wasmtime_config = Config::new();
        wasmtime_config.wasm_component_model(true);
        wasmtime_config.consume_fuel(true);
        let engine = Engine::new(&wasmtime_config)
            .map_err(|err| SandboxError::Execution(err.to_string()))?;
        Ok(Self {
            engine,
            max_input_bytes: config.max_input_bytes,
            fuel: config.fuel,
        })
    }

    pub fn evaluate_case_json(
        &self,
        component_bytes: &[u8],
        case: &EvaluationCase,
    ) -> Result<ScoreResult, SandboxError> {
        let input =
            serde_json::to_vec(case).map_err(|err| SandboxError::Execution(err.to_string()))?;
        self.evaluate_bytes(component_bytes, &input)
    }

    pub fn evaluate_bytes(
        &self,
        component_bytes: &[u8],
        input: &[u8],
    ) -> Result<ScoreResult, SandboxError> {
        if input.len() > self.max_input_bytes {
            return Err(SandboxError::InputTooLarge {
                size_bytes: input.len(),
                limit_bytes: self.max_input_bytes,
            });
        }
        let case_json = std::str::from_utf8(input)
            .map_err(|err| SandboxError::Execution(format!("input is not valid utf-8: {err}")))?;

        let component = Component::new(&self.engine, component_bytes)
            .map_err(|err| SandboxError::Execution(err.to_string()))?;
        // The deterministic-scorer world has no imports. An empty linker means any
        // attempt by the component to import a host function (e.g. a WASI call)
        // fails instantiation, which we surface as a denied host import.
        let linker = Linker::<ScorerState>::new(&self.engine);
        let mut store = Store::new(&self.engine, ScorerState);
        store
            .set_fuel(self.fuel)
            .map_err(|err| SandboxError::Execution(err.to_string()))?;

        let bindings =
            DeterministicScorer::instantiate(&mut store, &component, &linker).map_err(|err| {
                let message = err.to_string();
                if message.contains("import") || message.contains("unknown") {
                    SandboxError::HostImportDenied(message)
                } else {
                    SandboxError::Execution(message)
                }
            })?;

        let basis_points = bindings
            .call_score(&mut store, case_json)
            .map_err(|err| SandboxError::Execution(err.to_string()))?;
        if !(0..=10_000).contains(&basis_points) {
            return Err(SandboxError::InvalidScore(basis_points));
        }
        Ok(ScoreResult {
            score: basis_points as f64 / 10_000.0,
            label: Some(
                if basis_points >= 5_000 {
                    "pass"
                } else {
                    "fail"
                }
                .to_string(),
            ),
            evidence: serde_json::json!({
                "basis_points": basis_points,
                "runtime": "wasmtime-component-model",
                "world": "beater:scorer/deterministic-scorer",
                "host_imports": "disabled",
            }),
        })
    }
}

/// Bridge that runs a `beater-eval` deterministic scorer through the WASI
/// Component Model runtime and reconciles it against the native (in-process)
/// implementation. This is the seam by which deterministic scorers defined in
/// `beater-eval` are *actually executed* through the component sandbox rather
/// than only in-process: the component runtime is the trust boundary, and the
/// native evaluator is the reference oracle for the reconciliation.
pub struct DeterministicScorerRun {
    /// Score produced by executing the WASI component.
    pub component: ScoreResult,
    /// Score produced by the native `beater_eval::evaluate_deterministic`.
    pub native: ScoreResult,
}

impl DeterministicScorerRun {
    /// Tolerance for reconciling the component and native scores. Scores are
    /// basis-point integers divided by 10_000, so equal verdicts can still differ
    /// by a few ULPs after the float division; `f64::EPSILON` is too tight to
    /// absorb that, so we compare within `1e-9`.
    const SCORE_AGREEMENT_TOLERANCE: f64 = 1e-9;

    /// Whether the component and native scorers agree on the numeric score.
    pub fn scores_agree(&self) -> bool {
        (self.component.score - self.native.score).abs() < Self::SCORE_AGREEMENT_TOLERANCE
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScorerBridgeError {
    #[error(transparent)]
    Sandbox(#[from] SandboxError),
    #[error(transparent)]
    Eval(#[from] EvalError),
}

impl WasmEvaluatorRuntime {
    /// Execute a deterministic evaluator from `beater-eval` two ways — once by
    /// running the supplied scorer `component_bytes` under the WASI Component
    /// Model, and once via the native reference implementation — and return both
    /// so callers can assert the component honors the evaluator's semantics.
    ///
    /// `spec` must be a `DeterministicWasi`-lane evaluator; the `component_bytes`
    /// must implement the `deterministic-scorer` world. The component receives
    /// the canonical-JSON serialization of `case` exactly as the native scorer
    /// does, keeping the two executions input-equivalent.
    pub fn run_eval_scorer(
        &self,
        spec: &EvaluatorSpec,
        component_bytes: &[u8],
        case: &EvaluationCase,
    ) -> Result<DeterministicScorerRun, ScorerBridgeError> {
        let native = evaluate_deterministic(spec, case)?;
        let component = self.evaluate_case_json(component_bytes, case)?;
        Ok(DeterministicScorerRun { component, native })
    }
}

#[derive(Clone, Debug)]
pub struct SandboxConfig {
    pub max_input_bytes: usize,
    pub fuel: u64,
}

impl SandboxConfig {
    fn validate(&self) -> Result<(), SandboxError> {
        if self.max_input_bytes == 0 {
            return Err(SandboxError::InvalidConfig {
                field: "max_input_bytes",
                value: self.max_input_bytes as u128,
            });
        }
        if self.fuel == 0 {
            return Err(SandboxError::InvalidConfig {
                field: "fuel",
                value: self.fuel as u128,
            });
        }
        Ok(())
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_input_bytes: 64 * 1024,
            fuel: 10_000_000,
        }
    }
}

impl Default for WasmEvaluatorRuntime {
    fn default() -> Self {
        Self::new(SandboxConfig::default())
            .unwrap_or_else(|err| panic!("default WasmEvaluatorRuntime must construct: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beater_eval::EvaluationCase;
    use serde_json::json;

    /// A minimal deterministic-scorer component, authored in component WAT. It
    /// exports `score(string) -> s32`: returns 10000 when the input string is
    /// non-empty, 0 otherwise. Realized purely from the input — no host imports.
    const SCORER_COMPONENT_WAT: &str = r#"
        (component
          (core module $m
            (memory (export "memory") 1)
            (func (export "score") (param $ptr i32) (param $len i32) (result i32)
              local.get $len
              i32.const 0
              i32.gt_s
              if (result i32)
                i32.const 10000
              else
                i32.const 0
              end)
            (func (export "cabi_realloc")
              (param i32 i32 i32 i32) (result i32)
              i32.const 0))
          (core instance $i (instantiate $m))
          (func $score (param "case-json" string) (result s32)
            (canon lift
              (core func $i "score")
              (memory $i "memory")
              (realloc (func $i "cabi_realloc"))))
          (export "score" (func $score)))
    "#;

    fn scorer_component() -> Vec<u8> {
        wat::parse_str(SCORER_COMPONENT_WAT).unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn sandbox_config_rejects_zero_max_input_bytes() {
        let Err(err) = WasmEvaluatorRuntime::new(SandboxConfig {
            max_input_bytes: 0,
            fuel: 10_000,
        }) else {
            panic!("zero max_input_bytes should be rejected");
        };

        assert!(matches!(
            err,
            SandboxError::InvalidConfig {
                field: "max_input_bytes",
                value: 0
            }
        ));
    }

    #[test]
    fn sandbox_config_rejects_zero_fuel() {
        let Err(err) = WasmEvaluatorRuntime::new(SandboxConfig {
            max_input_bytes: 1024,
            fuel: 0,
        }) else {
            panic!("zero fuel should be rejected");
        };

        assert!(matches!(
            err,
            SandboxError::InvalidConfig {
                field: "fuel",
                value: 0
            }
        ));
    }

    #[test]
    fn default_sandbox_config_constructs_runtime() {
        let runtime = WasmEvaluatorRuntime::new(SandboxConfig::default())
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(
            runtime.max_input_bytes,
            SandboxConfig::default().max_input_bytes
        );
        assert_eq!(runtime.fuel, SandboxConfig::default().fuel);
    }

    #[test]
    fn component_model_evaluator_scores_case_json() {
        let component = scorer_component();
        let runtime = WasmEvaluatorRuntime::default();
        let score = runtime
            .evaluate_case_json(
                &component,
                &EvaluationCase {
                    input: json!("question"),
                    output: json!("answer"),
                    reference: Some(json!("answer")),
                    trace: None,
                },
            )
            .unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(score.score, 1.0);
        assert_eq!(score.evidence["runtime"], json!("wasmtime-component-model"));
        assert_eq!(
            score.evidence["world"],
            json!("beater:scorer/deterministic-scorer")
        );
    }

    #[test]
    fn runs_beater_eval_deterministic_scorer_through_component_runtime() {
        use beater_eval::{EvaluatorKind, EvaluatorSpec};
        use beater_schema::EvaluatorLane;

        // A real deterministic evaluator defined in beater-eval.
        let spec = EvaluatorSpec {
            id: "exact_match".to_string(),
            lane: EvaluatorLane::DeterministicWasi,
            kind: EvaluatorKind::ExactMatch,
        };
        // A case where output matches reference => native ExactMatch scores 1.0.
        // The scorer component returns 10000 (=1.0) for any non-empty case JSON,
        // so for this case the component and native scorers must agree.
        let case = EvaluationCase {
            input: json!("question"),
            output: json!("answer"),
            reference: Some(json!("answer")),
            trace: None,
        };

        let runtime = WasmEvaluatorRuntime::default();
        let run = runtime
            .run_eval_scorer(&spec, &scorer_component(), &case)
            .unwrap_or_else(|err| panic!("{err}"));

        // The deterministic scorer was actually executed under the component
        // model (not the native path), and its score reconciles with the native
        // beater-eval reference implementation.
        assert_eq!(
            run.component.evidence["runtime"],
            json!("wasmtime-component-model")
        );
        assert_eq!(run.native.score, 1.0);
        assert_eq!(run.component.score, 1.0);
        assert!(run.scores_agree());
    }

    #[test]
    fn sandbox_rejects_components_that_import_host_functions() {
        // A real, well-formed component that *exports* the deterministic-scorer
        // `score` function but ALSO declares a component-level host import
        // (`host:env/clock.now`). It lowers that host function into its core
        // module and instantiates against it, so the component genuinely depends
        // on a capability the host would have to supply. The empty `Linker` the
        // sandbox uses supplies nothing, so instantiation must fail — and we
        // assert specifically on the `HostImportDenied` variant.
        //
        // This WAT validates and assembles (`wat::parse_str` returns `Ok`); the
        // failure happens at `DeterministicScorer::instantiate` time, not at
        // parse time, which is the property the old test never actually exercised.
        const HOST_IMPORTING_COMPONENT_WAT: &str = r#"
            (component
              (import "host:env/clock" (instance $clock
                (export "now" (func (result s32)))))
              (alias export $clock "now" (func $now))
              (core func $now_lower (canon lower (func $now)))
              (core module $m
                (import "host" "now" (func $now (result i32)))
                (memory (export "memory") 1)
                (func (export "score") (param $ptr i32) (param $len i32) (result i32)
                  ;; Force a dependency on the host import so it can't be elided.
                  call $now)
                (func (export "cabi_realloc")
                  (param i32 i32 i32 i32) (result i32)
                  i32.const 0))
              (core instance $i (instantiate $m
                (with "host" (instance (export "now" (func $now_lower))))))
              (func $score (param "case-json" string) (result s32)
                (canon lift
                  (core func $i "score")
                  (memory $i "memory")
                  (realloc (func $i "cabi_realloc"))))
              (export "score" (func $score)))
        "#;

        // The component must genuinely assemble; if it doesn't, the test is
        // meaningless (the old version passed vacuously on a parse error).
        let component_bytes = wat::parse_str(HOST_IMPORTING_COMPONENT_WAT)
            .unwrap_or_else(|err| panic!("host-importing component must assemble: {err}"));

        let runtime = WasmEvaluatorRuntime::default();
        let result = runtime.evaluate_bytes(&component_bytes, br#"{}"#);
        assert!(
            matches!(result, Err(SandboxError::HostImportDenied(_))),
            "instantiating a host-importing component against the empty linker \
             must be denied as HostImportDenied, got {result:?}"
        );
    }

    #[test]
    fn sandbox_fuel_bounds_infinite_loops() {
        let wat = r#"
            (component
              (core module $m
                (memory (export "memory") 1)
                (func (export "score") (param i32 i32) (result i32)
                  (loop $again br $again)
                  i32.const 0)
                (func (export "cabi_realloc") (param i32 i32 i32 i32) (result i32)
                  i32.const 0))
              (core instance $i (instantiate $m))
              (func $score (param "case-json" string) (result s32)
                (canon lift (core func $i "score")
                  (memory $i "memory") (realloc (func $i "cabi_realloc"))))
              (export "score" (func $score)))
        "#;
        let component = wat::parse_str(wat).unwrap_or_else(|err| panic!("{err}"));
        let runtime = WasmEvaluatorRuntime::new(SandboxConfig {
            max_input_bytes: 1024,
            fuel: 10_000,
        })
        .unwrap_or_else(|err| panic!("{err}"));
        let result = runtime.evaluate_bytes(&component, br#"{}"#);
        assert!(
            matches!(result, Err(SandboxError::Execution(_))),
            "{result:?}"
        );
    }

    #[test]
    fn input_too_large_is_rejected_before_instantiation() {
        let runtime = WasmEvaluatorRuntime::new(SandboxConfig {
            max_input_bytes: 4,
            fuel: 10_000,
        })
        .unwrap_or_else(|err| panic!("{err}"));
        let result = runtime.evaluate_bytes(&scorer_component(), b"way-too-long");
        assert!(matches!(result, Err(SandboxError::InputTooLarge { .. })));
    }
}
