# ReviewQueue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AnnotationSchema** | **interface{}** |  |
**CreatedAt** | **time.Time** |  |
**Name** | **string** |  |
**ProjectId** | **string** |  |
**QueueId** | **string** |  |
**TenantId** | **string** |  |

## Methods

### NewReviewQueue

`func NewReviewQueue(annotationSchema interface{}, createdAt time.Time, name string, projectId string, queueId string, tenantId string, ) *ReviewQueue`

NewReviewQueue instantiates a new ReviewQueue object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewReviewQueueWithDefaults

`func NewReviewQueueWithDefaults() *ReviewQueue`

NewReviewQueueWithDefaults instantiates a new ReviewQueue object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAnnotationSchema

`func (o *ReviewQueue) GetAnnotationSchema() interface{}`

GetAnnotationSchema returns the AnnotationSchema field if non-nil, zero value otherwise.

### GetAnnotationSchemaOk

`func (o *ReviewQueue) GetAnnotationSchemaOk() (*interface{}, bool)`

GetAnnotationSchemaOk returns a tuple with the AnnotationSchema field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAnnotationSchema

`func (o *ReviewQueue) SetAnnotationSchema(v interface{})`

SetAnnotationSchema sets AnnotationSchema field to given value.


### SetAnnotationSchemaNil

`func (o *ReviewQueue) SetAnnotationSchemaNil(b bool)`

 SetAnnotationSchemaNil sets the value for AnnotationSchema to be an explicit nil

### UnsetAnnotationSchema
`func (o *ReviewQueue) UnsetAnnotationSchema()`

UnsetAnnotationSchema ensures that no value is present for AnnotationSchema, not even an explicit nil
### GetCreatedAt

`func (o *ReviewQueue) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *ReviewQueue) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *ReviewQueue) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetName

`func (o *ReviewQueue) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *ReviewQueue) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *ReviewQueue) SetName(v string)`

SetName sets Name field to given value.


### GetProjectId

`func (o *ReviewQueue) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ReviewQueue) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ReviewQueue) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetQueueId

`func (o *ReviewQueue) GetQueueId() string`

GetQueueId returns the QueueId field if non-nil, zero value otherwise.

### GetQueueIdOk

`func (o *ReviewQueue) GetQueueIdOk() (*string, bool)`

GetQueueIdOk returns a tuple with the QueueId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetQueueId

`func (o *ReviewQueue) SetQueueId(v string)`

SetQueueId sets QueueId field to given value.


### GetTenantId

`func (o *ReviewQueue) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ReviewQueue) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ReviewQueue) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
