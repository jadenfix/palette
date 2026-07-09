# \IngestApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**ingest_period_drain_trace_ingested**](IngestApi.md#ingest_period_drain_trace_ingested) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain |
[**ingest_period_drain_trace_writes**](IngestApi.md#ingest_period_drain_trace_writes) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-writes/drain |
[**ingest_period_get_ingest_queue_status**](IngestApi.md#ingest_period_get_ingest_queue_status) | **GET** /v1/ingest/{tenant_id}/{project_id}/queue |
[**ingest_period_import_source**](IngestApi.md#ingest_period_import_source) | **POST** /v1/import/{tenant_id}/{project_id}/{environment_id} |
[**ingest_period_ingest_native**](IngestApi.md#ingest_period_ingest_native) | **POST** /v1/traces/native |
[**ingest_period_ingest_otlp**](IngestApi.md#ingest_period_ingest_otlp) | **POST** /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces |
[**ingest_period_ingest_otlp_json_collector**](IngestApi.md#ingest_period_ingest_otlp_json_collector) | **POST** /v1/traces |
[**ingest_period_reconcile_trace**](IngestApi.md#ingest_period_reconcile_trace) | **POST** /v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile |
[**ingest_period_replay_dead_letter**](IngestApi.md#ingest_period_replay_dead_letter) | **POST** /v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay |



## ingest_period_drain_trace_ingested

> models::TraceIngestedDrainReport ingest_period_drain_trace_ingested(tenant_id, project_id, limit, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**limit** | Option<**i32**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::TraceIngestedDrainReport**](TraceIngestedDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_drain_trace_writes

> models::TraceWriteDrainReport ingest_period_drain_trace_writes(tenant_id, project_id, limit, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**limit** | Option<**i32**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::TraceWriteDrainReport**](TraceWriteDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_get_ingest_queue_status

> models::IngestQueueStatus ingest_period_get_ingest_queue_status(tenant_id, project_id, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::IngestQueueStatus**](IngestQueueStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_import_source

> models::IngestOutcome ingest_period_import_source(tenant_id, project_id, environment_id, import_source_http_request, durability, authorization, x_beater_api_key)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**environment_id** | **String** | environment_id | [required] |
**import_source_http_request** | [**ImportSourceHttpRequest**](ImportSourceHttpRequest.md) |  | [required] |
**durability** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |

### Return type

[**models::IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_ingest_native

> models::IngestOutcome ingest_period_ingest_native(native_ingest_request, durability, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**native_ingest_request** | [**NativeIngestRequest**](NativeIngestRequest.md) |  | [required] |
**durability** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_ingest_otlp

> models::OtlpIngestOutcome ingest_period_ingest_otlp(tenant_id, project_id, environment_id, durability, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**environment_id** | **String** | environment_id | [required] |
**durability** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::OtlpIngestOutcome**](OtlpIngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_ingest_otlp_json_collector

> models::OtlpIngestOutcome ingest_period_ingest_otlp_json_collector(durability, authorization, x_beater_api_key, x_beater_tenant_id, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**durability** | Option<**String**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_tenant_id** | Option<**String**> | Tenant scope override for collector-style OTLP JSON |  |
**x_beater_project_id** | Option<**String**> | Project scope override for collector-style OTLP JSON |  |
**x_beater_environment_id** | Option<**String**> | Environment scope override for collector-style OTLP JSON |  |

### Return type

[**models::OtlpIngestOutcome**](OtlpIngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_reconcile_trace

> models::TraceIngestedReconcileReport ingest_period_reconcile_trace(tenant_id, project_id, trace_id, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**trace_id** | **String** | trace_id | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::TraceIngestedReconcileReport**](TraceIngestedReconcileReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## ingest_period_replay_dead_letter

> models::DeadLetterReplayReport ingest_period_replay_dead_letter(tenant_id, project_id, message_id, reset_attempts, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**message_id** | **String** | message_id | [required] |
**reset_attempts** | Option<**bool**> |  |  |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::DeadLetterReplayReport**](DeadLetterReplayReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
