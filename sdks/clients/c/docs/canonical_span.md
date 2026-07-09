# canonical_span_t

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attributes** | **list_t*** |  |
**cost** | [**money_t**](money.md) \* |  | [optional]
**end_time** | **char \*** |  | [optional]
**environment_id** | **char \*** |  |
**input_ref** | [**artifact_ref_t**](artifact_ref.md) \* |  | [optional]
**kind** | **char \*** | Canonical agent span kind such as agent.run or llm.call |
**model** | [**model_ref_t**](model_ref.md) \* |  | [optional]
**name** | **char \*** |  |
**normalizer_version** | **char \*** |  |
**output_ref** | [**artifact_ref_t**](artifact_ref.md) \* |  | [optional]
**parent_span_id** | **char \*** |  | [optional]
**project_id** | **char \*** |  |
**raw_ref** | [**artifact_ref_t**](artifact_ref.md) \* |  |
**schema_version** | **int** |  |
**seq** | **long** |  |
**span_id** | **char \*** |  |
**start_time** | **char \*** |  |
**status** | **span_status_t \*** |  |
**tenant_id** | **char \*** |  |
**tokens** | [**token_counts_t**](token_counts.md) \* |  | [optional]
**trace_id** | **char \*** |  |
**unmapped_attrs** | **any_type_t \*** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
