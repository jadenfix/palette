# BeaterClient::HealthApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
| ------ | ------------ | ----------- |
| [**health**](HealthApi.md#health) | **GET** /health |  |


## health

> <HealthResponse> health



### Examples

```ruby
require 'time'
require 'beater_client'

api_instance = BeaterClient::HealthApi.new

begin

  result = api_instance.health
  p result
rescue BeaterClient::ApiError => e
  puts "Error when calling HealthApi->health: #{e}"
end
```

#### Using the health_with_http_info variant

This returns an Array which contains the response data, status code and headers.

> <Array(<HealthResponse>, Integer, Hash)> health_with_http_info

```ruby
begin

  data, status_code, headers = api_instance.health_with_http_info
  p status_code # => 2xx
  p headers # => { ... }
  p data # => <HealthResponse>
rescue BeaterClient::ApiError => e
  puts "Error when calling HealthApi->health_with_http_info: #{e}"
end
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**HealthResponse**](HealthResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json
