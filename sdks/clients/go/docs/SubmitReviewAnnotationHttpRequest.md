# SubmitReviewAnnotationHttpRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AnnotationId** | Pointer to **NullableString** |  | [optional]
**Payload** | **interface{}** |  |
**ReviewerId** | **string** |  |
**Verdict** | [**ReviewVerdict**](ReviewVerdict.md) |  |

## Methods

### NewSubmitReviewAnnotationHttpRequest

`func NewSubmitReviewAnnotationHttpRequest(payload interface{}, reviewerId string, verdict ReviewVerdict, ) *SubmitReviewAnnotationHttpRequest`

NewSubmitReviewAnnotationHttpRequest instantiates a new SubmitReviewAnnotationHttpRequest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewSubmitReviewAnnotationHttpRequestWithDefaults

`func NewSubmitReviewAnnotationHttpRequestWithDefaults() *SubmitReviewAnnotationHttpRequest`

NewSubmitReviewAnnotationHttpRequestWithDefaults instantiates a new SubmitReviewAnnotationHttpRequest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAnnotationId

`func (o *SubmitReviewAnnotationHttpRequest) GetAnnotationId() string`

GetAnnotationId returns the AnnotationId field if non-nil, zero value otherwise.

### GetAnnotationIdOk

`func (o *SubmitReviewAnnotationHttpRequest) GetAnnotationIdOk() (*string, bool)`

GetAnnotationIdOk returns a tuple with the AnnotationId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAnnotationId

`func (o *SubmitReviewAnnotationHttpRequest) SetAnnotationId(v string)`

SetAnnotationId sets AnnotationId field to given value.

### HasAnnotationId

`func (o *SubmitReviewAnnotationHttpRequest) HasAnnotationId() bool`

HasAnnotationId returns a boolean if a field has been set.

### SetAnnotationIdNil

`func (o *SubmitReviewAnnotationHttpRequest) SetAnnotationIdNil(b bool)`

 SetAnnotationIdNil sets the value for AnnotationId to be an explicit nil

### UnsetAnnotationId
`func (o *SubmitReviewAnnotationHttpRequest) UnsetAnnotationId()`

UnsetAnnotationId ensures that no value is present for AnnotationId, not even an explicit nil
### GetPayload

`func (o *SubmitReviewAnnotationHttpRequest) GetPayload() interface{}`

GetPayload returns the Payload field if non-nil, zero value otherwise.

### GetPayloadOk

`func (o *SubmitReviewAnnotationHttpRequest) GetPayloadOk() (*interface{}, bool)`

GetPayloadOk returns a tuple with the Payload field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPayload

`func (o *SubmitReviewAnnotationHttpRequest) SetPayload(v interface{})`

SetPayload sets Payload field to given value.


### SetPayloadNil

`func (o *SubmitReviewAnnotationHttpRequest) SetPayloadNil(b bool)`

 SetPayloadNil sets the value for Payload to be an explicit nil

### UnsetPayload
`func (o *SubmitReviewAnnotationHttpRequest) UnsetPayload()`

UnsetPayload ensures that no value is present for Payload, not even an explicit nil
### GetReviewerId

`func (o *SubmitReviewAnnotationHttpRequest) GetReviewerId() string`

GetReviewerId returns the ReviewerId field if non-nil, zero value otherwise.

### GetReviewerIdOk

`func (o *SubmitReviewAnnotationHttpRequest) GetReviewerIdOk() (*string, bool)`

GetReviewerIdOk returns a tuple with the ReviewerId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReviewerId

`func (o *SubmitReviewAnnotationHttpRequest) SetReviewerId(v string)`

SetReviewerId sets ReviewerId field to given value.


### GetVerdict

`func (o *SubmitReviewAnnotationHttpRequest) GetVerdict() ReviewVerdict`

GetVerdict returns the Verdict field if non-nil, zero value otherwise.

### GetVerdictOk

`func (o *SubmitReviewAnnotationHttpRequest) GetVerdictOk() (*ReviewVerdict, bool)`

GetVerdictOk returns a tuple with the Verdict field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVerdict

`func (o *SubmitReviewAnnotationHttpRequest) SetVerdict(v ReviewVerdict)`

SetVerdict sets Verdict field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
