# JudgeBrokerOutcome

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Audit** | [**JudgeAuditRecord**](JudgeAuditRecord.md) |  |
**RemainingBudget** | [**Money**](Money.md) |  |
**Result** | [**ScoreResult**](ScoreResult.md) |  |

## Methods

### NewJudgeBrokerOutcome

`func NewJudgeBrokerOutcome(audit JudgeAuditRecord, remainingBudget Money, result ScoreResult, ) *JudgeBrokerOutcome`

NewJudgeBrokerOutcome instantiates a new JudgeBrokerOutcome object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewJudgeBrokerOutcomeWithDefaults

`func NewJudgeBrokerOutcomeWithDefaults() *JudgeBrokerOutcome`

NewJudgeBrokerOutcomeWithDefaults instantiates a new JudgeBrokerOutcome object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAudit

`func (o *JudgeBrokerOutcome) GetAudit() JudgeAuditRecord`

GetAudit returns the Audit field if non-nil, zero value otherwise.

### GetAuditOk

`func (o *JudgeBrokerOutcome) GetAuditOk() (*JudgeAuditRecord, bool)`

GetAuditOk returns a tuple with the Audit field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAudit

`func (o *JudgeBrokerOutcome) SetAudit(v JudgeAuditRecord)`

SetAudit sets Audit field to given value.


### GetRemainingBudget

`func (o *JudgeBrokerOutcome) GetRemainingBudget() Money`

GetRemainingBudget returns the RemainingBudget field if non-nil, zero value otherwise.

### GetRemainingBudgetOk

`func (o *JudgeBrokerOutcome) GetRemainingBudgetOk() (*Money, bool)`

GetRemainingBudgetOk returns a tuple with the RemainingBudget field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRemainingBudget

`func (o *JudgeBrokerOutcome) SetRemainingBudget(v Money)`

SetRemainingBudget sets RemainingBudget field to given value.


### GetResult

`func (o *JudgeBrokerOutcome) GetResult() ScoreResult`

GetResult returns the Result field if non-nil, zero value otherwise.

### GetResultOk

`func (o *JudgeBrokerOutcome) GetResultOk() (*ScoreResult, bool)`

GetResultOk returns a tuple with the Result field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResult

`func (o *JudgeBrokerOutcome) SetResult(v ScoreResult)`

SetResult sets Result field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
