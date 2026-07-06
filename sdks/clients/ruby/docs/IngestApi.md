# BeaterClient::IngestApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**drain_trace_ingested**](IngestApi.md#drain_trace_ingested) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-ingested/drain |  |
| [**drain_trace_writes**](IngestApi.md#drain_trace_writes) | **POST** /v1/ingest/{tenant_id}/{project_id}/trace-writes/drain |  |
| [**get_ingest_queue_status**](IngestApi.md#get_ingest_queue_status) | **GET** /v1/ingest/{tenant_id}/{project_id}/queue |  |
| [**import_source**](IngestApi.md#import_source) | **POST** /v1/import/{tenant_id}/{project_id}/{environment_id} |  |
| [**ingest_native**](IngestApi.md#ingest_native) | **POST** /v1/traces/native |  |
| [**ingest_otlp**](IngestApi.md#ingest_otlp) | **POST** /v1/otlp/{tenant_id}/{project_id}/{environment_id}/v1/traces |  |
| [**reconcile_trace**](IngestApi.md#reconcile_trace) | **POST** /v1/ingest/{tenant_id}/{project_id}/traces/{trace_id}/reconcile |  |
| [**replay_dead_letter**](IngestApi.md#replay_dead_letter) | **POST** /v1/ingest/{tenant_id}/{project_id}/dead-letters/{message_id}/replay |  |


## drain_trace_ingested

> <TraceIngestedDrainReport> drain_trace_ingested(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  limit: 56, # Integer |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.drain_trace_ingested(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->drain_trace_ingested: #{e}"
end
```

#### Using the drain_trace_ingested_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<TraceIngestedDrainReport>, Integer, Hash)> drain_trace_ingested_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.drain_trace_ingested_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <TraceIngestedDrainReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->drain_trace_ingested_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **limit** | **Integer** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**TraceIngestedDrainReport**](TraceIngestedDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## drain_trace_writes

> <TraceWriteDrainReport> drain_trace_writes(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  limit: 56, # Integer |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.drain_trace_writes(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->drain_trace_writes: #{e}"
end
```

#### Using the drain_trace_writes_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<TraceWriteDrainReport>, Integer, Hash)> drain_trace_writes_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.drain_trace_writes_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <TraceWriteDrainReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->drain_trace_writes_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **limit** | **Integer** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**TraceWriteDrainReport**](TraceWriteDrainReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## get_ingest_queue_status

> <IngestQueueStatus> get_ingest_queue_status(tenant_id, project_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.get_ingest_queue_status(tenant_id, project_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->get_ingest_queue_status: #{e}"
end
```

#### Using the get_ingest_queue_status_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<IngestQueueStatus>, Integer, Hash)> get_ingest_queue_status_with_http_info(tenant_id, project_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.get_ingest_queue_status_with_http_info(tenant_id, project_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <IngestQueueStatus>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->get_ingest_queue_status_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**IngestQueueStatus**](IngestQueueStatus.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## import_source

> <IngestOutcome> import_source(tenant_id, project_id, environment_id, import_source_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
environment_id = 'environment_id_example' # String | environment_id
import_source_http_request = BeaterClient::ImportSourceHttpRequest.new({source: 'source_example'}) # ImportSourceHttpRequest |
opts = {
  durability: 'durability_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example' # String | API key alternative for strict auth
}

begin

  result = api_instance.import_source(tenant_id, project_id, environment_id, import_source_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->import_source: #{e}"
end
```

#### Using the import_source_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<IngestOutcome>, Integer, Hash)> import_source_with_http_info(tenant_id, project_id, environment_id, import_source_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.import_source_with_http_info(tenant_id, project_id, environment_id, import_source_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <IngestOutcome>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->import_source_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **environment_id** | **String** | environment_id |  |
| **import_source_http_request** | [**ImportSourceHttpRequest**](ImportSourceHttpRequest.md) |  |  |
| **durability** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## ingest_native

> <IngestOutcome> ingest_native(native_ingest_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
native_ingest_request = BeaterClient::NativeIngestRequest.new({attributes: { key: 3.56}, kind: 'kind_example', name: 'name_example', redaction_class: BeaterClient::RedactionClass::PUBLIC, scope: BeaterClient::TenantScope.new({environment_id: 'environment_id_example', project_id: 'project_id_example', tenant_id: 'tenant_id_example'}), seq: 3.56, span_id: 'span_id_example', status: BeaterClient::SpanStatus::OK, trace_id: 'trace_id_example'}) # NativeIngestRequest |
opts = {
  durability: 'durability_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.ingest_native(native_ingest_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->ingest_native: #{e}"
end
```

#### Using the ingest_native_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<IngestOutcome>, Integer, Hash)> ingest_native_with_http_info(native_ingest_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.ingest_native_with_http_info(native_ingest_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <IngestOutcome>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->ingest_native_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **native_ingest_request** | [**NativeIngestRequest**](NativeIngestRequest.md) |  |  |
| **durability** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**IngestOutcome**](IngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json


## ingest_otlp

> <OtlpIngestOutcome> ingest_otlp(tenant_id, project_id, environment_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
environment_id = 'environment_id_example' # String | environment_id
opts = {
  durability: 'durability_example', # String |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.ingest_otlp(tenant_id, project_id, environment_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->ingest_otlp: #{e}"
end
```

#### Using the ingest_otlp_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<OtlpIngestOutcome>, Integer, Hash)> ingest_otlp_with_http_info(tenant_id, project_id, environment_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.ingest_otlp_with_http_info(tenant_id, project_id, environment_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <OtlpIngestOutcome>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->ingest_otlp_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **environment_id** | **String** | environment_id |  |
| **durability** | **String** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**OtlpIngestOutcome**](OtlpIngestOutcome.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## reconcile_trace

> <TraceIngestedReconcileReport> reconcile_trace(tenant_id, project_id, trace_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
trace_id = 'trace_id_example' # String | trace_id
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.reconcile_trace(tenant_id, project_id, trace_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->reconcile_trace: #{e}"
end
```

#### Using the reconcile_trace_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<TraceIngestedReconcileReport>, Integer, Hash)> reconcile_trace_with_http_info(tenant_id, project_id, trace_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.reconcile_trace_with_http_info(tenant_id, project_id, trace_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <TraceIngestedReconcileReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->reconcile_trace_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **trace_id** | **String** | trace_id |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**TraceIngestedReconcileReport**](TraceIngestedReconcileReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json


## replay_dead_letter

> <DeadLetterReplayReport> replay_dead_letter(tenant_id, project_id, message_id, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::IngestApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
message_id = 'message_id_example' # String | message_id
opts = {
  reset_attempts: true, # Boolean |
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.replay_dead_letter(tenant_id, project_id, message_id, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->replay_dead_letter: #{e}"
end
```

#### Using the replay_dead_letter_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<DeadLetterReplayReport>, Integer, Hash)> replay_dead_letter_with_http_info(tenant_id, project_id, message_id, opts)

```ruby
begin

  data, status_code, headers = api_instance.replay_dead_letter_with_http_info(tenant_id, project_id, message_id, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <DeadLetterReplayReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling IngestApi->replay_dead_letter_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **message_id** | **String** | message_id |  |
| **reset_attempts** | **Boolean** |  | [optional] |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**DeadLetterReplayReport**](DeadLetterReplayReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
