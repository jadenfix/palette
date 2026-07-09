# IngestAPI

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**IngestAPI_ingestDrainTraceIngested**](IngestAPI.md#IngestAPI_ingestDrainTraceIngested) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain |
[**IngestAPI_ingestDrainTraceWrites**](IngestAPI.md#IngestAPI_ingestDrainTraceWrites) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-writes/drain |
[**IngestAPI_ingestGetIngestQueueStatus**](IngestAPI.md#IngestAPI_ingestGetIngestQueueStatus) | **GET** /v1/ingest/{tenant_id}/{project_id}/queue |
[**IngestAPI_ingestImportSource**](IngestAPI.md#IngestAPI_ingestImportSource) | **POST** /v1/import/{tenant_id}/{project_id}/{environment_id} |
[**IngestAPI_ingestIngestNative**](IngestAPI.md#IngestAPI_ingestIngestNative) | **POST** /v1/traces/native |
[**IngestAPI_ingestIngestOtlp**](IngestAPI.md#IngestAPI_ingestIngestOtlp) | **POST** /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces |
[**IngestAPI_ingestIngestOtlpJsonCollector**](IngestAPI.md#IngestAPI_ingestIngestOtlpJsonCollector) | **POST** /v1/traces |
[**IngestAPI_ingestReconcileTrace**](IngestAPI.md#IngestAPI_ingestReconcileTrace) | **POST** /v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile |
[**IngestAPI_ingestReplayDeadLetter**](IngestAPI.md#IngestAPI_ingestReplayDeadLetter) | **POST** /v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay |


# **IngestAPI_ingestDrainTraceIngested**
```c
trace_ingested_drain_report_t* IngestAPI_ingestDrainTraceIngested(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**limit** | **int \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[trace_ingested_drain_report_t](trace_ingested_drain_report.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestDrainTraceWrites**
```c
trace_write_drain_report_t* IngestAPI_ingestDrainTraceWrites(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**limit** | **int \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[trace_write_drain_report_t](trace_write_drain_report.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestGetIngestQueueStatus**
```c
ingest_queue_status_t* IngestAPI_ingestGetIngestQueueStatus(apiClient_t *apiClient, char *tenant_id, char *project_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[ingest_queue_status_t](ingest_queue_status.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestImportSource**
```c
ingest_outcome_t* IngestAPI_ingestImportSource(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, import_source_http_request_t *import_source_http_request, char *durability, char *authorization, char *x_beater_api_key);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**environment_id** | **char \*** | environment_id |
**import_source_http_request** | **[import_source_http_request_t](import_source_http_request.md) \*** |  |
**durability** | **char \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]

### Return type

[ingest_outcome_t](ingest_outcome.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestIngestNative**
```c
ingest_outcome_t* IngestAPI_ingestIngestNative(apiClient_t *apiClient, native_ingest_request_t *native_ingest_request, char *durability, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**native_ingest_request** | **[native_ingest_request_t](native_ingest_request.md) \*** |  |
**durability** | **char \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[ingest_outcome_t](ingest_outcome.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestIngestOtlp**
```c
otlp_ingest_outcome_t* IngestAPI_ingestIngestOtlp(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, char *durability, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**environment_id** | **char \*** | environment_id |
**durability** | **char \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[otlp_ingest_outcome_t](otlp_ingest_outcome.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestIngestOtlpJsonCollector**
```c
otlp_ingest_outcome_t* IngestAPI_ingestIngestOtlpJsonCollector(apiClient_t *apiClient, char *durability, char *authorization, char *x_beater_api_key, char *x_beater_tenant_id, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**durability** | **char \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_tenant_id** | **char \*** | Tenant scope override for collector-style OTLP JSON | [optional]
**x_beater_project_id** | **char \*** | Project scope override for collector-style OTLP JSON | [optional]
**x_beater_environment_id** | **char \*** | Environment scope override for collector-style OTLP JSON | [optional]

### Return type

[otlp_ingest_outcome_t](otlp_ingest_outcome.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestReconcileTrace**
```c
trace_ingested_reconcile_report_t* IngestAPI_ingestReconcileTrace(apiClient_t *apiClient, char *tenant_id, char *project_id, char *trace_id, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**trace_id** | **char \*** | trace_id |
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[trace_ingested_reconcile_report_t](trace_ingested_reconcile_report.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **IngestAPI_ingestReplayDeadLetter**
```c
dead_letter_replay_report_t* IngestAPI_ingestReplayDeadLetter(apiClient_t *apiClient, char *tenant_id, char *project_id, char *message_id, int *reset_attempts, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);
```

### Parameters
Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
**apiClient** | **apiClient_t \*** | context containing the client configuration |
**tenant_id** | **char \*** | tenant_id |
**project_id** | **char \*** | project_id |
**message_id** | **char \*** | message_id |
**reset_attempts** | **int \*** |  | [optional]
**authorization** | **char \*** | Bearer API token for strict auth | [optional]
**x_beater_api_key** | **char \*** | API key alternative for strict auth | [optional]
**x_beater_project_id** | **char \*** | Strict-auth project scope | [optional]
**x_beater_environment_id** | **char \*** | Strict-auth environment scope | [optional]

### Return type

[dead_letter_replay_report_t](dead_letter_replay_report.md) *


### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
