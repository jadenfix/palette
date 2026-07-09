# EnqueueReviewTaskFromTraceHttpRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**DatasetCaseId** | Pointer to **NullableString** |  | [optional]
**DatasetId** | Pointer to **NullableString** |  | [optional]
**Priority** | Pointer to **NullableInt64** |  | [optional]
**SpanId** | Pointer to **NullableString** |  | [optional]
**TaskId** | Pointer to **NullableString** |  | [optional]
**TraceId** | **string** |  |

## Methods

### NewEnqueueReviewTaskFromTraceHttpRequest

`func NewEnqueueReviewTaskFromTraceHttpRequest(traceId string, ) *EnqueueReviewTaskFromTraceHttpRequest`

NewEnqueueReviewTaskFromTraceHttpRequest instantiates a new EnqueueReviewTaskFromTraceHttpRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewEnqueueReviewTaskFromTraceHttpRequestWithDefaults

`func NewEnqueueReviewTaskFromTraceHttpRequestWithDefaults() *EnqueueReviewTaskFromTraceHttpRequest`

NewEnqueueReviewTaskFromTraceHttpRequestWithDefaults instantiates a new EnqueueReviewTaskFromTraceHttpRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDatasetCaseId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetDatasetCaseId() string`

GetDatasetCaseId returns the DatasetCaseId field if non-nil, zero value otherwise.

### GetDatasetCaseIdOk

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetDatasetCaseIdOk() (*string, bool)`

GetDatasetCaseIdOk returns a tuple with the DatasetCaseId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetCaseId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetDatasetCaseId(v string)`

SetDatasetCaseId sets DatasetCaseId field to given value.

### HasDatasetCaseId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) HasDatasetCaseId() bool`

HasDatasetCaseId returns a boolean if a field has been set.

### SetDatasetCaseIdNil

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetDatasetCaseIdNil(b bool)`

 SetDatasetCaseIdNil sets the value for DatasetCaseId to be an explicit nil

### UnsetDatasetCaseId
`func (o *EnqueueReviewTaskFromTraceHttpRequest) UnsetDatasetCaseId()`

UnsetDatasetCaseId ensures that no value is present for DatasetCaseId, not even an explicit nil
### GetDatasetId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.

### HasDatasetId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) HasDatasetId() bool`

HasDatasetId returns a boolean if a field has been set.

### SetDatasetIdNil

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetDatasetIdNil(b bool)`

 SetDatasetIdNil sets the value for DatasetId to be an explicit nil

### UnsetDatasetId
`func (o *EnqueueReviewTaskFromTraceHttpRequest) UnsetDatasetId()`

UnsetDatasetId ensures that no value is present for DatasetId, not even an explicit nil
### GetPriority

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetPriority() int64`

GetPriority returns the Priority field if non-nil, zero value otherwise.

### GetPriorityOk

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetPriorityOk() (*int64, bool)`

GetPriorityOk returns a tuple with the Priority field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPriority

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetPriority(v int64)`

SetPriority sets Priority field to given value.

### HasPriority

`func (o *EnqueueReviewTaskFromTraceHttpRequest) HasPriority() bool`

HasPriority returns a boolean if a field has been set.

### SetPriorityNil

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetPriorityNil(b bool)`

 SetPriorityNil sets the value for Priority to be an explicit nil

### UnsetPriority
`func (o *EnqueueReviewTaskFromTraceHttpRequest) UnsetPriority()`

UnsetPriority ensures that no value is present for Priority, not even an explicit nil
### GetSpanId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.

### HasSpanId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) HasSpanId() bool`

HasSpanId returns a boolean if a field has been set.

### SetSpanIdNil

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetSpanIdNil(b bool)`

 SetSpanIdNil sets the value for SpanId to be an explicit nil

### UnsetSpanId
`func (o *EnqueueReviewTaskFromTraceHttpRequest) UnsetSpanId()`

UnsetSpanId ensures that no value is present for SpanId, not even an explicit nil
### GetTaskId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetTaskId() string`

GetTaskId returns the TaskId field if non-nil, zero value otherwise.

### GetTaskIdOk

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetTaskIdOk() (*string, bool)`

GetTaskIdOk returns a tuple with the TaskId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTaskId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetTaskId(v string)`

SetTaskId sets TaskId field to given value.

### HasTaskId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) HasTaskId() bool`

HasTaskId returns a boolean if a field has been set.

### SetTaskIdNil

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetTaskIdNil(b bool)`

 SetTaskIdNil sets the value for TaskId to be an explicit nil

### UnsetTaskId
`func (o *EnqueueReviewTaskFromTraceHttpRequest) UnsetTaskId()`

UnsetTaskId ensures that no value is present for TaskId, not even an explicit nil
### GetTraceId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *EnqueueReviewTaskFromTraceHttpRequest) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *EnqueueReviewTaskFromTraceHttpRequest) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
