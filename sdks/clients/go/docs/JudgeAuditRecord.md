# JudgeAuditRecord

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Cached** | **bool** |  |
**ChargedCost** | [**Money**](Money.md) |  |
**CreatedAt** | **time.Time** |  |
**EvaluatorId** | **string** |  |
**JudgeCallId** | **string** |  |
**Model** | **string** |  |
**ProjectId** | **string** |  |
**Provider** | **string** |  |
**ProviderCost** | [**Money**](Money.md) |  |
**ProviderSecretId** | **string** |  |
**RequestHash** | **string** |  |
**ResponseHash** | **string** |  |
**Score** | **float64** |  |
**TenantId** | **string** |  |

## Methods

### NewJudgeAuditRecord

`func NewJudgeAuditRecord(cached bool, chargedCost Money, createdAt time.Time, evaluatorId string, judgeCallId string, model string, projectId string, provider string, providerCost Money, providerSecretId string, requestHash string, responseHash string, score float64, tenantId string, ) *JudgeAuditRecord`

NewJudgeAuditRecord instantiates a new JudgeAuditRecord object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewJudgeAuditRecordWithDefaults

`func NewJudgeAuditRecordWithDefaults() *JudgeAuditRecord`

NewJudgeAuditRecordWithDefaults instantiates a new JudgeAuditRecord object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCached

`func (o *JudgeAuditRecord) GetCached() bool`

GetCached returns the Cached field if non-nil, zero value otherwise.

### GetCachedOk

`func (o *JudgeAuditRecord) GetCachedOk() (*bool, bool)`

GetCachedOk returns a tuple with the Cached field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCached

`func (o *JudgeAuditRecord) SetCached(v bool)`

SetCached sets Cached field to given value.


### GetChargedCost

`func (o *JudgeAuditRecord) GetChargedCost() Money`

GetChargedCost returns the ChargedCost field if non-nil, zero value otherwise.

### GetChargedCostOk

`func (o *JudgeAuditRecord) GetChargedCostOk() (*Money, bool)`

GetChargedCostOk returns a tuple with the ChargedCost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetChargedCost

`func (o *JudgeAuditRecord) SetChargedCost(v Money)`

SetChargedCost sets ChargedCost field to given value.


### GetCreatedAt

`func (o *JudgeAuditRecord) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *JudgeAuditRecord) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *JudgeAuditRecord) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetEvaluatorId

`func (o *JudgeAuditRecord) GetEvaluatorId() string`

GetEvaluatorId returns the EvaluatorId field if non-nil, zero value otherwise.

### GetEvaluatorIdOk

`func (o *JudgeAuditRecord) GetEvaluatorIdOk() (*string, bool)`

GetEvaluatorIdOk returns a tuple with the EvaluatorId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorId

`func (o *JudgeAuditRecord) SetEvaluatorId(v string)`

SetEvaluatorId sets EvaluatorId field to given value.


### GetJudgeCallId

`func (o *JudgeAuditRecord) GetJudgeCallId() string`

GetJudgeCallId returns the JudgeCallId field if non-nil, zero value otherwise.

### GetJudgeCallIdOk

`func (o *JudgeAuditRecord) GetJudgeCallIdOk() (*string, bool)`

GetJudgeCallIdOk returns a tuple with the JudgeCallId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetJudgeCallId

`func (o *JudgeAuditRecord) SetJudgeCallId(v string)`

SetJudgeCallId sets JudgeCallId field to given value.


### GetModel

`func (o *JudgeAuditRecord) GetModel() string`

GetModel returns the Model field if non-nil, zero value otherwise.

### GetModelOk

`func (o *JudgeAuditRecord) GetModelOk() (*string, bool)`

GetModelOk returns a tuple with the Model field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetModel

`func (o *JudgeAuditRecord) SetModel(v string)`

SetModel sets Model field to given value.


### GetProjectId

`func (o *JudgeAuditRecord) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *JudgeAuditRecord) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *JudgeAuditRecord) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetProvider

`func (o *JudgeAuditRecord) GetProvider() string`

GetProvider returns the Provider field if non-nil, zero value otherwise.

### GetProviderOk

`func (o *JudgeAuditRecord) GetProviderOk() (*string, bool)`

GetProviderOk returns a tuple with the Provider field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProvider

`func (o *JudgeAuditRecord) SetProvider(v string)`

SetProvider sets Provider field to given value.


### GetProviderCost

`func (o *JudgeAuditRecord) GetProviderCost() Money`

GetProviderCost returns the ProviderCost field if non-nil, zero value otherwise.

### GetProviderCostOk

`func (o *JudgeAuditRecord) GetProviderCostOk() (*Money, bool)`

GetProviderCostOk returns a tuple with the ProviderCost field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProviderCost

`func (o *JudgeAuditRecord) SetProviderCost(v Money)`

SetProviderCost sets ProviderCost field to given value.


### GetProviderSecretId

`func (o *JudgeAuditRecord) GetProviderSecretId() string`

GetProviderSecretId returns the ProviderSecretId field if non-nil, zero value otherwise.

### GetProviderSecretIdOk

`func (o *JudgeAuditRecord) GetProviderSecretIdOk() (*string, bool)`

GetProviderSecretIdOk returns a tuple with the ProviderSecretId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProviderSecretId

`func (o *JudgeAuditRecord) SetProviderSecretId(v string)`

SetProviderSecretId sets ProviderSecretId field to given value.


### GetRequestHash

`func (o *JudgeAuditRecord) GetRequestHash() string`

GetRequestHash returns the RequestHash field if non-nil, zero value otherwise.

### GetRequestHashOk

`func (o *JudgeAuditRecord) GetRequestHashOk() (*string, bool)`

GetRequestHashOk returns a tuple with the RequestHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRequestHash

`func (o *JudgeAuditRecord) SetRequestHash(v string)`

SetRequestHash sets RequestHash field to given value.


### GetResponseHash

`func (o *JudgeAuditRecord) GetResponseHash() string`

GetResponseHash returns the ResponseHash field if non-nil, zero value otherwise.

### GetResponseHashOk

`func (o *JudgeAuditRecord) GetResponseHashOk() (*string, bool)`

GetResponseHashOk returns a tuple with the ResponseHash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResponseHash

`func (o *JudgeAuditRecord) SetResponseHash(v string)`

SetResponseHash sets ResponseHash field to given value.


### GetScore

`func (o *JudgeAuditRecord) GetScore() float64`

GetScore returns the Score field if non-nil, zero value otherwise.

### GetScoreOk

`func (o *JudgeAuditRecord) GetScoreOk() (*float64, bool)`

GetScoreOk returns a tuple with the Score field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScore

`func (o *JudgeAuditRecord) SetScore(v float64)`

SetScore sets Score field to given value.


### GetTenantId

`func (o *JudgeAuditRecord) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *JudgeAuditRecord) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *JudgeAuditRecord) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
