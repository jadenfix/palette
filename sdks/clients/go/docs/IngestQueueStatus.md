# IngestQueueStatus

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**DeadLetters** | [**[]DeadLetter**](DeadLetter.md) |  |
**ProjectId** | **string** |  |
**TenantId** | **string** |  |
**TotalDepth** | **int32** |  |
**TraceIngestedDepth** | **int32** |  |
**TraceWriteDepth** | **int32** |  |

## Methods

### NewIngestQueueStatus

`func NewIngestQueueStatus(deadLetters []DeadLetter, projectId string, tenantId string, totalDepth int32, traceIngestedDepth int32, traceWriteDepth int32, ) *IngestQueueStatus`

NewIngestQueueStatus instantiates a new IngestQueueStatus object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewIngestQueueStatusWithDefaults

`func NewIngestQueueStatusWithDefaults() *IngestQueueStatus`

NewIngestQueueStatusWithDefaults instantiates a new IngestQueueStatus object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDeadLetters

`func (o *IngestQueueStatus) GetDeadLetters() []DeadLetter`

GetDeadLetters returns the DeadLetters field if non-nil, zero value otherwise.

### GetDeadLettersOk

`func (o *IngestQueueStatus) GetDeadLettersOk() (*[]DeadLetter, bool)`

GetDeadLettersOk returns a tuple with the DeadLetters field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDeadLetters

`func (o *IngestQueueStatus) SetDeadLetters(v []DeadLetter)`

SetDeadLetters sets DeadLetters field to given value.


### GetProjectId

`func (o *IngestQueueStatus) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *IngestQueueStatus) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *IngestQueueStatus) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetTenantId

`func (o *IngestQueueStatus) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *IngestQueueStatus) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *IngestQueueStatus) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTotalDepth

`func (o *IngestQueueStatus) GetTotalDepth() int32`

GetTotalDepth returns the TotalDepth field if non-nil, zero value otherwise.

### GetTotalDepthOk

`func (o *IngestQueueStatus) GetTotalDepthOk() (*int32, bool)`

GetTotalDepthOk returns a tuple with the TotalDepth field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTotalDepth

`func (o *IngestQueueStatus) SetTotalDepth(v int32)`

SetTotalDepth sets TotalDepth field to given value.


### GetTraceIngestedDepth

`func (o *IngestQueueStatus) GetTraceIngestedDepth() int32`

GetTraceIngestedDepth returns the TraceIngestedDepth field if non-nil, zero value otherwise.

### GetTraceIngestedDepthOk

`func (o *IngestQueueStatus) GetTraceIngestedDepthOk() (*int32, bool)`

GetTraceIngestedDepthOk returns a tuple with the TraceIngestedDepth field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceIngestedDepth

`func (o *IngestQueueStatus) SetTraceIngestedDepth(v int32)`

SetTraceIngestedDepth sets TraceIngestedDepth field to given value.


### GetTraceWriteDepth

`func (o *IngestQueueStatus) GetTraceWriteDepth() int32`

GetTraceWriteDepth returns the TraceWriteDepth field if non-nil, zero value otherwise.

### GetTraceWriteDepthOk

`func (o *IngestQueueStatus) GetTraceWriteDepthOk() (*int32, bool)`

GetTraceWriteDepthOk returns a tuple with the TraceWriteDepth field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceWriteDepth

`func (o *IngestQueueStatus) SetTraceWriteDepth(v int32)`

SetTraceWriteDepth sets TraceWriteDepth field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
