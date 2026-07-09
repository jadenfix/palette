# CreateReviewQueueHttpRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AnnotationSchema** | **interface{}** |  |
**Name** | **string** |  |
**QueueId** | Pointer to **NullableString** |  | [optional]

## Methods

### NewCreateReviewQueueHttpRequest

`func NewCreateReviewQueueHttpRequest(annotationSchema interface{}, name string, ) *CreateReviewQueueHttpRequest`

NewCreateReviewQueueHttpRequest instantiates a new CreateReviewQueueHttpRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCreateReviewQueueHttpRequestWithDefaults

`func NewCreateReviewQueueHttpRequestWithDefaults() *CreateReviewQueueHttpRequest`

NewCreateReviewQueueHttpRequestWithDefaults instantiates a new CreateReviewQueueHttpRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAnnotationSchema

`func (o *CreateReviewQueueHttpRequest) GetAnnotationSchema() interface{}`

GetAnnotationSchema returns the AnnotationSchema field if non-nil, zero value otherwise.

### GetAnnotationSchemaOk

`func (o *CreateReviewQueueHttpRequest) GetAnnotationSchemaOk() (*interface{}, bool)`

GetAnnotationSchemaOk returns a tuple with the AnnotationSchema field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAnnotationSchema

`func (o *CreateReviewQueueHttpRequest) SetAnnotationSchema(v interface{})`

SetAnnotationSchema sets AnnotationSchema field to given value.


### SetAnnotationSchemaNil

`func (o *CreateReviewQueueHttpRequest) SetAnnotationSchemaNil(b bool)`

 SetAnnotationSchemaNil sets the value for AnnotationSchema to be an explicit nil

### UnsetAnnotationSchema
`func (o *CreateReviewQueueHttpRequest) UnsetAnnotationSchema()`

UnsetAnnotationSchema ensures that no value is present for AnnotationSchema, not even an explicit nil
### GetName

`func (o *CreateReviewQueueHttpRequest) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *CreateReviewQueueHttpRequest) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *CreateReviewQueueHttpRequest) SetName(v string)`

SetName sets Name field to given value.


### GetQueueId

`func (o *CreateReviewQueueHttpRequest) GetQueueId() string`

GetQueueId returns the QueueId field if non-nil, zero value otherwise.

### GetQueueIdOk

`func (o *CreateReviewQueueHttpRequest) GetQueueIdOk() (*string, bool)`

GetQueueIdOk returns a tuple with the QueueId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetQueueId

`func (o *CreateReviewQueueHttpRequest) SetQueueId(v string)`

SetQueueId sets QueueId field to given value.

### HasQueueId

`func (o *CreateReviewQueueHttpRequest) HasQueueId() bool`

HasQueueId returns a boolean if a field has been set.

### SetQueueIdNil

`func (o *CreateReviewQueueHttpRequest) SetQueueIdNil(b bool)`

 SetQueueIdNil sets the value for QueueId to be an explicit nil

### UnsetQueueId
`func (o *CreateReviewQueueHttpRequest) UnsetQueueId()`

UnsetQueueId ensures that no value is present for QueueId, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
