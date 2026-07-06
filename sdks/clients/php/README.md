# beater-client

Agent observability, evaluation, gating, and human-review APIs for Beater


## Installation & Usage

### Requirements

PHP 7.4 and later.
Should also work with PHP 8.0.

### Composer

To install the bindings via [Composer](https://getcomposer.org/), add the following to `composer.json`:

```json
{
  "repositories": [
    {
      "type": "vcs",
      "url": "https://github.com/GIT_USER_ID/GIT_REPO_ID.git"
    }
  ],
  "require": {
    "GIT_USER_ID/GIT_REPO_ID": "*@dev"
  }
}
```

Then run `composer install`

### Manual Installation

Download the files and include `autoload.php`:

```php
<?php
require_once('/path/to/beater-client/vendor/autoload.php');
```

## Getting Started

Please follow the [installation procedure](#installation--usage) and then run the following:

```php
<?php
require_once(__DIR__ . '/vendor/autoload.php');




$apiInstance = new Beater\Client\Api\AlertsApi(
    // If you want use custom http client, pass your client which implements `GuzzleHttp\ClientInterface`.
    // This is optional, `GuzzleHttp\Client` will be used as default.
    new GuzzleHttp\Client()
);
$tenant_id = 'tenant_id_example'; // string | tenant_id
$project_id = 'project_id_example'; // string | project_id
$trace_id = 'trace_id_example'; // string | trace_id
$evaluate_alert_request = new \Beater\Client\Model\EvaluateAlertRequest(); // \Beater\Client\Model\EvaluateAlertRequest
$authorization = 'authorization_example'; // string | Bearer API token for strict auth
$x_beater_api_key = 'x_beater_api_key_example'; // string | API key alternative for strict auth
$x_beater_project_id = 'x_beater_project_id_example'; // string | Strict-auth project scope
$x_beater_environment_id = 'x_beater_environment_id_example'; // string | Strict-auth environment scope

try {
    $result = $apiInstance->evaluateAlert($tenant_id, $project_id, $trace_id, $evaluate_alert_request, $authorization, $x_beater_api_key, $x_beater_project_id, $x_beater_environment_id);
    print_r($result);
} catch (Exception $e) {
    echo 'Exception when calling AlertsApi->evaluateAlert: ', $e->getMessage(), PHP_EOL;
}

```

## API Endpoints

All URIs are relative to *http://localhost*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*AlertsApi* | [**evaluateAlert**](docs/Api/AlertsApi.md#evaluatealert) | **POST** /v1/alerts/{tenant_id}/{project_id}/traces/{trace_id}/webhook |
*ApiKeysApi* | [**createApiKey**](docs/Api/ApiKeysApi.md#createapikey) | **POST** /v1/api-keys/{tenant_id}/{project_id}/{environment_id} |
*ApiKeysApi* | [**revokeApiKey**](docs/Api/ApiKeysApi.md#revokeapikey) | **POST** /v1/api-keys/{tenant_id}/{project_id}/{environment_id}/{api_key_id}/revoke |
*ArchiveApi* | [**archiveTrace**](docs/Api/ArchiveApi.md#archivetrace) | **POST** /v1/archive/{tenant_id}/{project_id}/{trace_id} |
*ArchiveApi* | [**queryArchiveSpans**](docs/Api/ArchiveApi.md#queryarchivespans) | **GET** /v1/archive/{tenant_id}/{project_id}/spans |
*AuditApi* | [**listAuditEvents**](docs/Api/AuditApi.md#listauditevents) | **GET** /v1/audit/{tenant_id}/{project_id} |
*CalibrationsApi* | [**runCalibration**](docs/Api/CalibrationsApi.md#runcalibration) | **POST** /v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id} |
*ConnectorsApi* | [**connectConnector**](docs/Api/ConnectorsApi.md#connectconnector) | **POST** /v1/connectors/{tenant_id}/{project_id}/connect |
*ConnectorsApi* | [**connectorStatus**](docs/Api/ConnectorsApi.md#connectorstatus) | **GET** /v1/connectors/{tenant_id}/{project_id}/status |
*ConnectorsApi* | [**getConnectorSkills**](docs/Api/ConnectorsApi.md#getconnectorskills) | **GET** /v1/connectors/{tenant_id}/{project_id}/skills |
*ConnectorsApi* | [**invokeConnectorTool**](docs/Api/ConnectorsApi.md#invokeconnectortool) | **POST** /v1/connectors/{tenant_id}/{project_id}/invoke |
*ConnectorsApi* | [**listConnectorTools**](docs/Api/ConnectorsApi.md#listconnectortools) | **GET** /v1/connectors/{tenant_id}/{project_id}/tools |
*ConnectorsApi* | [**listConnectors**](docs/Api/ConnectorsApi.md#listconnectors) | **GET** /v1/connectors/{tenant_id}/{project_id} |
*DatasetsApi* | [**createDataset**](docs/Api/DatasetsApi.md#createdataset) | **POST** /v1/datasets/{tenant_id}/{project_id} |
*DatasetsApi* | [**createDatasetVersion**](docs/Api/DatasetsApi.md#createdatasetversion) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions |
*DatasetsApi* | [**promoteDatasetCaseFromTrace**](docs/Api/DatasetsApi.md#promotedatasetcasefromtrace) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/cases/from-trace |
*EvalsApi* | [**runDeterministicEval**](docs/Api/EvalsApi.md#rundeterministiceval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/deterministic |
*EvalsApi* | [**runJudgeEval**](docs/Api/EvalsApi.md#runjudgeeval) | **POST** /v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/evals/judge |
*ExperimentsApi* | [**runDeterministicExperiment**](docs/Api/ExperimentsApi.md#rundeterministicexperiment) | **POST** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/deterministic |
*ExperimentsApi* | [**runJudgeExperiment**](docs/Api/ExperimentsApi.md#runjudgeexperiment) | **POST** /v1/experiments/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id}/judge |
*GatesApi* | [**createGate**](docs/Api/GatesApi.md#creategate) | **POST** /v1/gates/{tenant_id}/{project_id} |
*GatesApi* | [**runGate**](docs/Api/GatesApi.md#rungate) | **POST** /v1/gates/{tenant_id}/{project_id}/{gate_id}/run |
*HealthApi* | [**health**](docs/Api/HealthApi.md#health) | **GET** /health |
*IngestApi* | [**drainTraceIngested**](docs/Api/IngestApi.md#draintraceingested) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain |
*IngestApi* | [**drainTraceWrites**](docs/Api/IngestApi.md#draintracewrites) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-writes/drain |
*IngestApi* | [**getIngestQueueStatus**](docs/Api/IngestApi.md#getingestqueuestatus) | **GET** /v1/ingest/{tenant_id}/{project_id}/queue |
*IngestApi* | [**importSource**](docs/Api/IngestApi.md#importsource) | **POST** /v1/import/{tenant_id}/{project_id}/{environment_id} |
*IngestApi* | [**ingestNative**](docs/Api/IngestApi.md#ingestnative) | **POST** /v1/traces/native |
*IngestApi* | [**ingestOtlp**](docs/Api/IngestApi.md#ingestotlp) | **POST** /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces |
*IngestApi* | [**reconcileTrace**](docs/Api/IngestApi.md#reconciletrace) | **POST** /v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile |
*IngestApi* | [**replayDeadLetter**](docs/Api/IngestApi.md#replaydeadletter) | **POST** /v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay |
*JudgeApi* | [**evaluateJudge**](docs/Api/JudgeApi.md#evaluatejudge) | **POST** /v1/judge/{tenant_id}/{project_id}/evaluate |
*JudgeApi* | [**listJudgeLedger**](docs/Api/JudgeApi.md#listjudgeledger) | **GET** /v1/judge/{tenant_id}/{project_id}/ledger |
*OnlineApi* | [**decideOnlineSampling**](docs/Api/OnlineApi.md#decideonlinesampling) | **POST** /v1/online/{tenant_id}/{project_id}/traces/{trace_id}/sampling |
*PromptsApi* | [**addPromptVersion**](docs/Api/PromptsApi.md#addpromptversion) | **POST** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions |
*PromptsApi* | [**createPrompt**](docs/Api/PromptsApi.md#createprompt) | **POST** /v1/prompts/{tenant_id}/{project_id} |
*PromptsApi* | [**diffPromptVersions**](docs/Api/PromptsApi.md#diffpromptversions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/diff |
*PromptsApi* | [**getPrompt**](docs/Api/PromptsApi.md#getprompt) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id} |
*PromptsApi* | [**listPromptVersions**](docs/Api/PromptsApi.md#listpromptversions) | **GET** /v1/prompts/{tenant_id}/{project_id}/{prompt_id}/versions |
*PromptsApi* | [**listPrompts**](docs/Api/PromptsApi.md#listprompts) | **GET** /v1/prompts/{tenant_id}/{project_id} |
*ProviderSecretsApi* | [**createProviderSecret**](docs/Api/ProviderSecretsApi.md#createprovidersecret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id} |
*ProviderSecretsApi* | [**listProviderSecrets**](docs/Api/ProviderSecretsApi.md#listprovidersecrets) | **GET** /v1/provider-secrets/{tenant_id}/{project_id} |
*ProviderSecretsApi* | [**revokeProviderSecret**](docs/Api/ProviderSecretsApi.md#revokeprovidersecret) | **POST** /v1/provider-secrets/{tenant_id}/{project_id}/{provider_secret_id}/revoke |
*ReviewsApi* | [**createReviewQueue**](docs/Api/ReviewsApi.md#createreviewqueue) | **POST** /v1/review-queues/{tenant_id}/{project_id} |
*ReviewsApi* | [**enqueueReviewTaskFromTrace**](docs/Api/ReviewsApi.md#enqueuereviewtaskfromtrace) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/from-trace |
*ReviewsApi* | [**listReviewTasks**](docs/Api/ReviewsApi.md#listreviewtasks) | **GET** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks |
*ReviewsApi* | [**promoteReviewAnnotation**](docs/Api/ReviewsApi.md#promotereviewannotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations/{annotation_id}/promote |
*ReviewsApi* | [**submitReviewAnnotation**](docs/Api/ReviewsApi.md#submitreviewannotation) | **POST** /v1/review-queues/{tenant_id}/{project_id}/{queue_id}/tasks/{task_id}/annotations |
*ScenariosApi* | [**createScenario**](docs/Api/ScenariosApi.md#createscenario) | **POST** /v1/scenarios/{tenant_id}/{project_id} |
*ScenariosApi* | [**getScenario**](docs/Api/ScenariosApi.md#getscenario) | **GET** /v1/scenarios/{tenant_id}/{project_id}/{scenario_id} |
*ScenariosApi* | [**listScenarios**](docs/Api/ScenariosApi.md#listscenarios) | **GET** /v1/scenarios/{tenant_id}/{project_id} |
*ScenariosApi* | [**mineScenarios**](docs/Api/ScenariosApi.md#minescenarios) | **POST** /v1/scenarios/{tenant_id}/{project_id}/mine |
*SearchApi* | [**searchSpans**](docs/Api/SearchApi.md#searchspans) | **GET** /v1/search/{tenant_id}/spans |
*SpansApi* | [**getSpan**](docs/Api/SpansApi.md#getspan) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id} |
*SpansApi* | [**getSpanIo**](docs/Api/SpansApi.md#getspanio) | **GET** /v1/spans/{tenant_id}/{trace_id}/{span_id}/io |
*TracesApi* | [**getTrace**](docs/Api/TracesApi.md#gettrace) | **GET** /v1/traces/{tenant_id}/{trace_id} |
*TracesApi* | [**listTraces**](docs/Api/TracesApi.md#listtraces) | **GET** /v1/traces/{tenant_id} |
*UsageApi* | [**getUsageSummary**](docs/Api/UsageApi.md#getusagesummary) | **GET** /v1/usage/{tenant_id}/{project_id} |

## Models

- [AddPromptVersionRequest](docs/Model/AddPromptVersionRequest.md)
- [AlertDecision](docs/Model/AlertDecision.md)
- [AlertInput](docs/Model/AlertInput.md)
- [AlertLinks](docs/Model/AlertLinks.md)
- [AlertPolicy](docs/Model/AlertPolicy.md)
- [AlertSeverity](docs/Model/AlertSeverity.md)
- [ApiKeyCreatedResponse](docs/Model/ApiKeyCreatedResponse.md)
- [ApiScope](docs/Model/ApiScope.md)
- [ArchiveManifest](docs/Model/ArchiveManifest.md)
- [ArchiveQueryResponse](docs/Model/ArchiveQueryResponse.md)
- [ArchivedSpanRow](docs/Model/ArchivedSpanRow.md)
- [ArtifactRef](docs/Model/ArtifactRef.md)
- [AuditAction](docs/Model/AuditAction.md)
- [AuditEvent](docs/Model/AuditEvent.md)
- [AuditOutcome](docs/Model/AuditOutcome.md)
- [AuthContext](docs/Model/AuthContext.md)
- [BusMessage](docs/Model/BusMessage.md)
- [CalibrationConfusion](docs/Model/CalibrationConfusion.md)
- [CalibrationItem](docs/Model/CalibrationItem.md)
- [CalibrationLabel](docs/Model/CalibrationLabel.md)
- [CalibrationPolicy](docs/Model/CalibrationPolicy.md)
- [CalibrationReport](docs/Model/CalibrationReport.md)
- [CanonicalSpan](docs/Model/CanonicalSpan.md)
- [CaseExperimentScore](docs/Model/CaseExperimentScore.md)
- [CaseOutputOverrideRequest](docs/Model/CaseOutputOverrideRequest.md)
- [ConnectConnectorRequest](docs/Model/ConnectConnectorRequest.md)
- [ConnectionLink](docs/Model/ConnectionLink.md)
- [ConnectionStatus](docs/Model/ConnectionStatus.md)
- [ConnectorSkillsResponse](docs/Model/ConnectorSkillsResponse.md)
- [ConnectorTool](docs/Model/ConnectorTool.md)
- [CreateApiKeyHttpRequest](docs/Model/CreateApiKeyHttpRequest.md)
- [CreateDatasetRequest](docs/Model/CreateDatasetRequest.md)
- [CreateDatasetVersionRequest](docs/Model/CreateDatasetVersionRequest.md)
- [CreateGateRequest](docs/Model/CreateGateRequest.md)
- [CreatePromptRequest](docs/Model/CreatePromptRequest.md)
- [CreateProviderSecretHttpRequest](docs/Model/CreateProviderSecretHttpRequest.md)
- [CreateReviewQueueHttpRequest](docs/Model/CreateReviewQueueHttpRequest.md)
- [CreateScenarioRequest](docs/Model/CreateScenarioRequest.md)
- [CreatedPrompt](docs/Model/CreatedPrompt.md)
- [Currency](docs/Model/Currency.md)
- [Dataset](docs/Model/Dataset.md)
- [DatasetCase](docs/Model/DatasetCase.md)
- [DatasetEvalReport](docs/Model/DatasetEvalReport.md)
- [DatasetVersionSnapshot](docs/Model/DatasetVersionSnapshot.md)
- [DeadLetter](docs/Model/DeadLetter.md)
- [DeadLetterReplayReport](docs/Model/DeadLetterReplayReport.md)
- [DiffLine](docs/Model/DiffLine.md)
- [DiffLineKind](docs/Model/DiffLineKind.md)
- [EnqueueReviewTaskFromTraceHttpRequest](docs/Model/EnqueueReviewTaskFromTraceHttpRequest.md)
- [ErrorResponse](docs/Model/ErrorResponse.md)
- [EvalReproducibility](docs/Model/EvalReproducibility.md)
- [EvalResult](docs/Model/EvalResult.md)
- [EvaluateAlertRequest](docs/Model/EvaluateAlertRequest.md)
- [EvaluationCase](docs/Model/EvaluationCase.md)
- [EvaluatorKind](docs/Model/EvaluatorKind.md)
- [EvaluatorKindOneOf](docs/Model/EvaluatorKindOneOf.md)
- [EvaluatorKindOneOf1](docs/Model/EvaluatorKindOneOf1.md)
- [EvaluatorKindOneOf10](docs/Model/EvaluatorKindOneOf10.md)
- [EvaluatorKindOneOf2](docs/Model/EvaluatorKindOneOf2.md)
- [EvaluatorKindOneOf3](docs/Model/EvaluatorKindOneOf3.md)
- [EvaluatorKindOneOf4](docs/Model/EvaluatorKindOneOf4.md)
- [EvaluatorKindOneOf5](docs/Model/EvaluatorKindOneOf5.md)
- [EvaluatorKindOneOf6](docs/Model/EvaluatorKindOneOf6.md)
- [EvaluatorKindOneOf7](docs/Model/EvaluatorKindOneOf7.md)
- [EvaluatorKindOneOf8](docs/Model/EvaluatorKindOneOf8.md)
- [EvaluatorKindOneOf9](docs/Model/EvaluatorKindOneOf9.md)
- [EvaluatorLane](docs/Model/EvaluatorLane.md)
- [EvaluatorSpec](docs/Model/EvaluatorSpec.md)
- [ExperimentComparison](docs/Model/ExperimentComparison.md)
- [ExperimentRunReport](docs/Model/ExperimentRunReport.md)
- [FailureMode](docs/Model/FailureMode.md)
- [GateDecision](docs/Model/GateDecision.md)
- [GateDefinition](docs/Model/GateDefinition.md)
- [GatePolicy](docs/Model/GatePolicy.md)
- [GateRunReport](docs/Model/GateRunReport.md)
- [HealthResponse](docs/Model/HealthResponse.md)
- [ImportSourceHttpRequest](docs/Model/ImportSourceHttpRequest.md)
- [InconclusivePolicy](docs/Model/InconclusivePolicy.md)
- [IngestOutcome](docs/Model/IngestOutcome.md)
- [IngestQueueStatus](docs/Model/IngestQueueStatus.md)
- [InvokeConnectorRequest](docs/Model/InvokeConnectorRequest.md)
- [JudgeAuditRecord](docs/Model/JudgeAuditRecord.md)
- [JudgeBrokerOutcome](docs/Model/JudgeBrokerOutcome.md)
- [ListScenariosResponse](docs/Model/ListScenariosResponse.md)
- [MaintenanceWindow](docs/Model/MaintenanceWindow.md)
- [MineScenariosRequest](docs/Model/MineScenariosRequest.md)
- [MineScenariosResponse](docs/Model/MineScenariosResponse.md)
- [ModelRef](docs/Model/ModelRef.md)
- [Money](docs/Model/Money.md)
- [NativeIngestRequest](docs/Model/NativeIngestRequest.md)
- [OnlineSamplingPolicy](docs/Model/OnlineSamplingPolicy.md)
- [OtlpIngestOutcome](docs/Model/OtlpIngestOutcome.md)
- [PageRunSummary](docs/Model/PageRunSummary.md)
- [PageRunSummaryItemsInner](docs/Model/PageRunSummaryItemsInner.md)
- [PerturbationKnobs](docs/Model/PerturbationKnobs.md)
- [PromoteReviewAnnotationHttpRequest](docs/Model/PromoteReviewAnnotationHttpRequest.md)
- [PromoteTraceCaseRequest](docs/Model/PromoteTraceCaseRequest.md)
- [Prompt](docs/Model/Prompt.md)
- [PromptListResponse](docs/Model/PromptListResponse.md)
- [PromptTemplate](docs/Model/PromptTemplate.md)
- [PromptVariable](docs/Model/PromptVariable.md)
- [PromptVersion](docs/Model/PromptVersion.md)
- [PromptVersionDiff](docs/Model/PromptVersionDiff.md)
- [PromptVersionListResponse](docs/Model/PromptVersionListResponse.md)
- [PromptVersionMetadata](docs/Model/PromptVersionMetadata.md)
- [ProviderSecretMetadata](docs/Model/ProviderSecretMetadata.md)
- [PublishAck](docs/Model/PublishAck.md)
- [QueuedTraceWork](docs/Model/QueuedTraceWork.md)
- [RedactionClass](docs/Model/RedactionClass.md)
- [ReliabilityBin](docs/Model/ReliabilityBin.md)
- [ReviewAnnotation](docs/Model/ReviewAnnotation.md)
- [ReviewQueue](docs/Model/ReviewQueue.md)
- [ReviewTask](docs/Model/ReviewTask.md)
- [ReviewTaskState](docs/Model/ReviewTaskState.md)
- [ReviewVerdict](docs/Model/ReviewVerdict.md)
- [RevokedApiKey](docs/Model/RevokedApiKey.md)
- [RevokedProviderSecret](docs/Model/RevokedProviderSecret.md)
- [RunCalibrationHttpRequest](docs/Model/RunCalibrationHttpRequest.md)
- [RunDeterministicEvalRequest](docs/Model/RunDeterministicEvalRequest.md)
- [RunExperimentRequest](docs/Model/RunExperimentRequest.md)
- [RunGateRequest](docs/Model/RunGateRequest.md)
- [RunJudgeDatasetEvalRequest](docs/Model/RunJudgeDatasetEvalRequest.md)
- [RunJudgeEvalHttpRequest](docs/Model/RunJudgeEvalHttpRequest.md)
- [RunJudgeExperimentRequest](docs/Model/RunJudgeExperimentRequest.md)
- [RunSummary](docs/Model/RunSummary.md)
- [SamplingDecision](docs/Model/SamplingDecision.md)
- [SamplingReason](docs/Model/SamplingReason.md)
- [Scenario](docs/Model/Scenario.md)
- [ScenarioCluster](docs/Model/ScenarioCluster.md)
- [ScoreResult](docs/Model/ScoreResult.md)
- [SearchHit](docs/Model/SearchHit.md)
- [SearchResponse](docs/Model/SearchResponse.md)
- [Signature](docs/Model/Signature.md)
- [SpanIoResponse](docs/Model/SpanIoResponse.md)
- [SpanIoValue](docs/Model/SpanIoValue.md)
- [SpanIoValueOneOf](docs/Model/SpanIoValueOneOf.md)
- [SpanIoValueOneOf1](docs/Model/SpanIoValueOneOf1.md)
- [SpanIoValueOneOf2](docs/Model/SpanIoValueOneOf2.md)
- [SpanIoValueOneOf3](docs/Model/SpanIoValueOneOf3.md)
- [SpanStatus](docs/Model/SpanStatus.md)
- [StatisticalTest](docs/Model/StatisticalTest.md)
- [SubmitReviewAnnotationHttpRequest](docs/Model/SubmitReviewAnnotationHttpRequest.md)
- [TenantScope](docs/Model/TenantScope.md)
- [TokenCounts](docs/Model/TokenCounts.md)
- [ToolExecution](docs/Model/ToolExecution.md)
- [Toolkit](docs/Model/Toolkit.md)
- [TraceIngestedDrainReport](docs/Model/TraceIngestedDrainReport.md)
- [TraceIngestedReconcileReport](docs/Model/TraceIngestedReconcileReport.md)
- [TraceView](docs/Model/TraceView.md)
- [TraceWriteDrainReport](docs/Model/TraceWriteDrainReport.md)
- [UsageSummary](docs/Model/UsageSummary.md)
- [UsageTotal](docs/Model/UsageTotal.md)
- [WebhookDelivery](docs/Model/WebhookDelivery.md)
- [WriteAck](docs/Model/WriteAck.md)

## Authorization
Endpoints do not require authorization.

## Tests

To run the tests, use:

```bash
composer install
vendor/bin/phpunit
```

## Author



## About this package

This PHP package is automatically generated by the [OpenAPI Generator](https://openapi-generator.tech) project:

- API version: `0.1.0`
    - Package version: `0.1.0`
    - Generator version: `7.11.0`
- Build package: `org.openapitools.codegen.languages.PhpClientCodegen`
