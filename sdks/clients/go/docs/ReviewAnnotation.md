# ReviewAnnotation

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AnnotationId** | **string** |  |
**CreatedAt** | **time.Time** |  |
**Payload** | **interface{}** |  |
**ProjectId** | **string** |  |
**QueueId** | **string** |  |
**ReviewerId** | **string** |  |
**TaskId** | **string** |  |
**TenantId** | **string** |  |
**Verdict** | [**ReviewVerdict**](ReviewVerdict.md) |  |

## Methods

### NewReviewAnnotation

`func NewReviewAnnotation(annotationId string, createdAt time.Time, payload interface{}, projectId string, queueId string, reviewerId string, taskId string, tenantId string, verdict ReviewVerdict, ) *ReviewAnnotation`

NewReviewAnnotation instantiates a new ReviewAnnotation object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewReviewAnnotationWithDefaults

`func NewReviewAnnotationWithDefaults() *ReviewAnnotation`

NewReviewAnnotationWithDefaults instantiates a new ReviewAnnotation object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAnnotationId

`func (o *ReviewAnnotation) GetAnnotationId() string`

GetAnnotationId returns the AnnotationId field if non-nil, zero value otherwise.

### GetAnnotationIdOk

`func (o *ReviewAnnotation) GetAnnotationIdOk() (*string, bool)`

GetAnnotationIdOk returns a tuple with the AnnotationId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAnnotationId

`func (o *ReviewAnnotation) SetAnnotationId(v string)`

SetAnnotationId sets AnnotationId field to given value.


### GetCreatedAt

`func (o *ReviewAnnotation) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *ReviewAnnotation) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *ReviewAnnotation) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetPayload

`func (o *ReviewAnnotation) GetPayload() interface{}`

GetPayload returns the Payload field if non-nil, zero value otherwise.

### GetPayloadOk

`func (o *ReviewAnnotation) GetPayloadOk() (*interface{}, bool)`

GetPayloadOk returns a tuple with the Payload field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPayload

`func (o *ReviewAnnotation) SetPayload(v interface{})`

SetPayload sets Payload field to given value.


### SetPayloadNil

`func (o *ReviewAnnotation) SetPayloadNil(b bool)`

 SetPayloadNil sets the value for Payload to be an explicit nil

### UnsetPayload
`func (o *ReviewAnnotation) UnsetPayload()`

UnsetPayload ensures that no value is present for Payload, not even an explicit nil
### GetProjectId

`func (o *ReviewAnnotation) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ReviewAnnotation) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ReviewAnnotation) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetQueueId

`func (o *ReviewAnnotation) GetQueueId() string`

GetQueueId returns the QueueId field if non-nil, zero value otherwise.

### GetQueueIdOk

`func (o *ReviewAnnotation) GetQueueIdOk() (*string, bool)`

GetQueueIdOk returns a tuple with the QueueId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetQueueId

`func (o *ReviewAnnotation) SetQueueId(v string)`

SetQueueId sets QueueId field to given value.


### GetReviewerId

`func (o *ReviewAnnotation) GetReviewerId() string`

GetReviewerId returns the ReviewerId field if non-nil, zero value otherwise.

### GetReviewerIdOk

`func (o *ReviewAnnotation) GetReviewerIdOk() (*string, bool)`

GetReviewerIdOk returns a tuple with the ReviewerId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReviewerId

`func (o *ReviewAnnotation) SetReviewerId(v string)`

SetReviewerId sets ReviewerId field to given value.


### GetTaskId

`func (o *ReviewAnnotation) GetTaskId() string`

GetTaskId returns the TaskId field if non-nil, zero value otherwise.

### GetTaskIdOk

`func (o *ReviewAnnotation) GetTaskIdOk() (*string, bool)`

GetTaskIdOk returns a tuple with the TaskId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTaskId

`func (o *ReviewAnnotation) SetTaskId(v string)`

SetTaskId sets TaskId field to given value.


### GetTenantId

`func (o *ReviewAnnotation) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ReviewAnnotation) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ReviewAnnotation) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetVerdict

`func (o *ReviewAnnotation) GetVerdict() ReviewVerdict`

GetVerdict returns the Verdict field if non-nil, zero value otherwise.

### GetVerdictOk

`func (o *ReviewAnnotation) GetVerdictOk() (*ReviewVerdict, bool)`

GetVerdictOk returns a tuple with the Verdict field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVerdict

`func (o *ReviewAnnotation) SetVerdict(v ReviewVerdict)`

SetVerdict sets Verdict field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
