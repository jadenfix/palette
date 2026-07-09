# \OnlineApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**online_period_decide_online_sampling**](OnlineApi.md#online_period_decide_online_sampling) | **POST** /v1/online/{tenant_id}/{project_id}/traces/{trace_id}/sampling |



## online_period_decide_online_sampling

> models::SamplingDecision online_period_decide_online_sampling(tenant_id, project_id, trace_id, online_sampling_policy, authorization, x_beater_api_key, x_beater_project_id, x_beater_environment_id)


### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**tenant_id** | **String** | tenant_id | [required] |
**project_id** | **String** | project_id | [required] |
**trace_id** | **String** | trace_id | [required] |
**online_sampling_policy** | [**OnlineSamplingPolicy**](OnlineSamplingPolicy.md) |  | [required] |
**authorization** | Option<**String**> | Bearer API token for strict auth |  |
**x_beater_api_key** | Option<**String**> | API key alternative for strict auth |  |
**x_beater_project_id** | Option<**String**> | Strict-auth project scope |  |
**x_beater_environment_id** | Option<**String**> | Strict-auth environment scope |  |

### Return type

[**models::SamplingDecision**](SamplingDecision.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
