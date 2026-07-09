# EvalResult

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Cost** | Pointer to [**NullableMoney**](Money.md) |  | [optional]
**CreatedAt** | **time.Time** |  |
**EvalResultId** | **string** |  |
**Evidence** | **interface{}** |  |
**Label** | Pointer to **NullableString** |  | [optional]
**NonReproducibleReason** | Pointer to **NullableString** |  | [optional]
**ProjectId** | **string** |  |
**Reproducibility** | [**EvalReproducibility**](EvalReproducibility.md) |  |
**Score** | **float64** |  |
**SpanId** | Pointer to **string** |  | [optional]
**TenantId** | **string** |  |
**Tokens** | Pointer to [**NullableTokenCounts**](TokenCounts.md) |  | [optional]
**TraceId** | **string** |  |

## Methods

### NewEvalResult

`func NewEvalResult(createdAt time.Time, evalResultId string, evidence interface{}, projectId string, reproducibility EvalReproducibility, score float64, tenantId string, traceId string, ) *EvalResult`

NewEvalResult instantiates a new EvalResult object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewEvalResultWithDefaults

`func NewEvalResultWithDefaults() *EvalResult`

NewEvalResultWithDefaults instantiates a new EvalResult object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCost

`func (o *EvalResult) GetCost() Money`

GetCost returns the Cost field if non-nil, zero value otherwise.

### GetCostOk

`func (o *EvalResult) GetCostOk() (*Money, bool)`

GetCostOk returns a tuple with the Cost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCost

`func (o *EvalResult) SetCost(v Money)`

SetCost sets Cost field to given value.

### HasCost

`func (o *EvalResult) HasCost() bool`

HasCost returns a boolean if a field has been set.

### SetCostNil

`func (o *EvalResult) SetCostNil(b bool)`

 SetCostNil sets the value for Cost to be an explicit nil

### UnsetCost
`func (o *EvalResult) UnsetCost()`

UnsetCost ensures that no value is present for Cost, not even an explicit nil
### GetCreatedAt

`func (o *EvalResult) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *EvalResult) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *EvalResult) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetEvalResultId

`func (o *EvalResult) GetEvalResultId() string`

GetEvalResultId returns the EvalResultId field if non-nil, zero value otherwise.

### GetEvalResultIdOk

`func (o *EvalResult) GetEvalResultIdOk() (*string, bool)`

GetEvalResultIdOk returns a tuple with the EvalResultId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvalResultId

`func (o *EvalResult) SetEvalResultId(v string)`

SetEvalResultId sets EvalResultId field to given value.


### GetEvidence

`func (o *EvalResult) GetEvidence() interface{}`

GetEvidence returns the Evidence field if non-nil, zero value otherwise.

### GetEvidenceOk

`func (o *EvalResult) GetEvidenceOk() (*interface{}, bool)`

GetEvidenceOk returns a tuple with the Evidence field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvidence

`func (o *EvalResult) SetEvidence(v interface{})`

SetEvidence sets Evidence field to given value.


### SetEvidenceNil

`func (o *EvalResult) SetEvidenceNil(b bool)`

 SetEvidenceNil sets the value for Evidence to be an explicit nil

### UnsetEvidence
`func (o *EvalResult) UnsetEvidence()`

UnsetEvidence ensures that no value is present for Evidence, not even an explicit nil
### GetLabel

`func (o *EvalResult) GetLabel() string`

GetLabel returns the Label field if non-nil, zero value otherwise.

### GetLabelOk

`func (o *EvalResult) GetLabelOk() (*string, bool)`

GetLabelOk returns a tuple with the Label field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLabel

`func (o *EvalResult) SetLabel(v string)`

SetLabel sets Label field to given value.

### HasLabel

`func (o *EvalResult) HasLabel() bool`

HasLabel returns a boolean if a field has been set.

### SetLabelNil

`func (o *EvalResult) SetLabelNil(b bool)`

 SetLabelNil sets the value for Label to be an explicit nil

### UnsetLabel
`func (o *EvalResult) UnsetLabel()`

UnsetLabel ensures that no value is present for Label, not even an explicit nil
### GetNonReproducibleReason

`func (o *EvalResult) GetNonReproducibleReason() string`

GetNonReproducibleReason returns the NonReproducibleReason field if non-nil, zero value otherwise.

### GetNonReproducibleReasonOk

`func (o *EvalResult) GetNonReproducibleReasonOk() (*string, bool)`

GetNonReproducibleReasonOk returns a tuple with the NonReproducibleReason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNonReproducibleReason

`func (o *EvalResult) SetNonReproducibleReason(v string)`

SetNonReproducibleReason sets NonReproducibleReason field to given value.

### HasNonReproducibleReason

`func (o *EvalResult) HasNonReproducibleReason() bool`

HasNonReproducibleReason returns a boolean if a field has been set.

### SetNonReproducibleReasonNil

`func (o *EvalResult) SetNonReproducibleReasonNil(b bool)`

 SetNonReproducibleReasonNil sets the value for NonReproducibleReason to be an explicit nil

### UnsetNonReproducibleReason
`func (o *EvalResult) UnsetNonReproducibleReason()`

UnsetNonReproducibleReason ensures that no value is present for NonReproducibleReason, not even an explicit nil
### GetProjectId

`func (o *EvalResult) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *EvalResult) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *EvalResult) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetReproducibility

`func (o *EvalResult) GetReproducibility() EvalReproducibility`

GetReproducibility returns the Reproducibility field if non-nil, zero value otherwise.

### GetReproducibilityOk

`func (o *EvalResult) GetReproducibilityOk() (*EvalReproducibility, bool)`

GetReproducibilityOk returns a tuple with the Reproducibility field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReproducibility

`func (o *EvalResult) SetReproducibility(v EvalReproducibility)`

SetReproducibility sets Reproducibility field to given value.


### GetScore

`func (o *EvalResult) GetScore() float64`

GetScore returns the Score field if non-nil, zero value otherwise.

### GetScoreOk

`func (o *EvalResult) GetScoreOk() (*float64, bool)`

GetScoreOk returns a tuple with the Score field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScore

`func (o *EvalResult) SetScore(v float64)`

SetScore sets Score field to given value.


### GetSpanId

`func (o *EvalResult) GetSpanId() string`

GetSpanId returns the SpanId field if non-nil, zero value otherwise.

### GetSpanIdOk

`func (o *EvalResult) GetSpanIdOk() (*string, bool)`

GetSpanIdOk returns a tuple with the SpanId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanId

`func (o *EvalResult) SetSpanId(v string)`

SetSpanId sets SpanId field to given value.

### HasSpanId

`func (o *EvalResult) HasSpanId() bool`

HasSpanId returns a boolean if a field has been set.

### GetTenantId

`func (o *EvalResult) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *EvalResult) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *EvalResult) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTokens

`func (o *EvalResult) GetTokens() TokenCounts`

GetTokens returns the Tokens field if non-nil, zero value otherwise.

### GetTokensOk

`func (o *EvalResult) GetTokensOk() (*TokenCounts, bool)`

GetTokensOk returns a tuple with the Tokens field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTokens

`func (o *EvalResult) SetTokens(v TokenCounts)`

SetTokens sets Tokens field to given value.

### HasTokens

`func (o *EvalResult) HasTokens() bool`

HasTokens returns a boolean if a field has been set.

### SetTokensNil

`func (o *EvalResult) SetTokensNil(b bool)`

 SetTokensNil sets the value for Tokens to be an explicit nil

### UnsetTokens
`func (o *EvalResult) UnsetTokens()`

UnsetTokens ensures that no value is present for Tokens, not even an explicit nil
### GetTraceId

`func (o *EvalResult) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *EvalResult) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *EvalResult) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
