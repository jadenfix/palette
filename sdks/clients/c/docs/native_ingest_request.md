# native_ingest_request_t

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attributes** | **list_t*** |  |
**auth_context** | [**auth_context_t**](auth_context.md) \* |  | [optional]
**cost** | [**money_t**](money.md) \* |  | [optional]
**end_time** | **char \*** |  | [optional]
**idempotency_key** | **char \*** |  | [optional]
**input** | **any_type_t \*** |  | [optional]
**kind** | **char \*** | Canonical agent span kind such as agent.run or llm.call |
**model** | [**model_ref_t**](model_ref.md) \* |  | [optional]
**name** | **char \*** |  |
**output** | **any_type_t \*** |  | [optional]
**parent_span_id** | **char \*** |  | [optional]
**redaction_class** | **redaction_class_t \*** |  |
**scope** | [**tenant_scope_t**](tenant_scope.md) \* |  |
**seq** | **long** |  |
**span_id** | **char \*** |  |
**start_time** | **char \*** |  | [optional]
**status** | **span_status_t \*** |  |
**tokens** | [**token_counts_t**](token_counts.md) \* |  | [optional]
**trace_id** | **char \*** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
