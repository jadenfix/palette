# BeaterClient::CalibrationsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**run_calibration**](CalibrationsApi.md#run_calibration) | **POST** /v1/calibrations/{tenant_id}/{project_id}/{dataset_id}/versions/{version_id} |  |


## run_calibration

> <CalibrationReport> run_calibration(tenant_id, project_id, dataset_id, version_id, run_calibration_http_request, opts)



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::CalibrationsApi.new
tenant_id = 'tenant_id_example' # String | tenant_id
project_id = 'project_id_example' # String | project_id
dataset_id = 'dataset_id_example' # String | dataset_id
version_id = 'version_id_example' # String | version_id
run_calibration_http_request = BeaterClient::RunCalibrationHttpRequest.new # RunCalibrationHttpRequest |
opts = {
  authorization: 'authorization_example', # String | Bearer API token for strict auth
  x_beater_api_key: 'x_beater_api_key_example', # String | API key alternative for strict auth
  x_beater_project_id: 'x_beater_project_id_example', # String | Strict-auth project scope
  x_beater_environment_id: 'x_beater_environment_id_example' # String | Strict-auth environment scope
}

begin

  result = api_instance.run_calibration(tenant_id, project_id, dataset_id, version_id, run_calibration_http_request, opts)
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling CalibrationsApi->run_calibration: #{e}"
end
```

#### Using the run_calibration_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<CalibrationReport>, Integer, Hash)> run_calibration_with_http_info(tenant_id, project_id, dataset_id, version_id, run_calibration_http_request, opts)

```ruby
begin

  data, status_code, headers = api_instance.run_calibration_with_http_info(tenant_id, project_id, dataset_id, version_id, run_calibration_http_request, opts)
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <CalibrationReport>
rescue BeaterClient::ApiError => e
  puts "Error when calling CalibrationsApi->run_calibration_with_http_info: #{e}"
end
```

### Parameters

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **tenant_id** | **String** | tenant_id |  |
| **project_id** | **String** | project_id |  |
| **dataset_id** | **String** | dataset_id |  |
| **version_id** | **String** | version_id |  |
| **run_calibration_http_request** | [**RunCalibrationHttpRequest**](RunCalibrationHttpRequest.md) |  |  |
| **authorization** | **String** | Bearer API token for strict auth | [optional] |
| **x_beater_api_key** | **String** | API key alternative for strict auth | [optional] |
| **x_beater_project_id** | **String** | Strict-auth project scope | [optional] |
| **x_beater_environment_id** | **String** | Strict-auth environment scope | [optional] |

### Return type

[**CalibrationReport**](CalibrationReport.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json
