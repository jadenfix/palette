# Scenario

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CreatedAt** | **time.Time** | When the scenario was created. |
**ExemplarTraceId** | **string** |  |
**ExpectedOutcome** | Pointer to **NullableString** | Expected outcome for replay assertions, if known. | [optional]
**FailureMode** | [**FailureMode**](FailureMode.md) | The dominant failure mode this scenario reproduces. |
**PerturbationKnobs** | [**PerturbationKnobs**](PerturbationKnobs.md) | Suggested perturbation knobs for replay. |
**RecurrenceCount** | **int32** | How many traces exhibited this scenario. |
**RedactionClass** | [**RedactionClass**](RedactionClass.md) | Redaction classification of the scenario payload. |
**ScenarioId** | **string** | Stable, deterministic identifier for the scenario. |
**Scope** | [**TenantScope**](TenantScope.md) | Tenant/project/environment scope this scenario belongs to. |
**SourceTraceIds** | **[]string** | Trace ids the scenario was mined from, sorted ascending. |
**Title** | **string** | Human-readable title. |

## Methods

### NewScenario

`func NewScenario(createdAt time.Time, exemplarTraceId string, failureMode FailureMode, perturbationKnobs PerturbationKnobs, recurrenceCount int32, redactionClass RedactionClass, scenarioId string, scope TenantScope, sourceTraceIds []string, title string, ) *Scenario`

NewScenario instantiates a new Scenario object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewScenarioWithDefaults

`func NewScenarioWithDefaults() *Scenario`

NewScenarioWithDefaults instantiates a new Scenario object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCreatedAt

`func (o *Scenario) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *Scenario) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *Scenario) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetExemplarTraceId

`func (o *Scenario) GetExemplarTraceId() string`

GetExemplarTraceId returns the ExemplarTraceId field if non-nil, zero value otherwise.

### GetExemplarTraceIdOk

`func (o *Scenario) GetExemplarTraceIdOk() (*string, bool)`

GetExemplarTraceIdOk returns a tuple with the ExemplarTraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExemplarTraceId

`func (o *Scenario) SetExemplarTraceId(v string)`

SetExemplarTraceId sets ExemplarTraceId field to given value.


### GetExpectedOutcome

`func (o *Scenario) GetExpectedOutcome() string`

GetExpectedOutcome returns the ExpectedOutcome field if non-nil, zero value otherwise.

### GetExpectedOutcomeOk

`func (o *Scenario) GetExpectedOutcomeOk() (*string, bool)`

GetExpectedOutcomeOk returns a tuple with the ExpectedOutcome field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExpectedOutcome

`func (o *Scenario) SetExpectedOutcome(v string)`

SetExpectedOutcome sets ExpectedOutcome field to given value.

### HasExpectedOutcome

`func (o *Scenario) HasExpectedOutcome() bool`

HasExpectedOutcome returns a boolean if a field has been set.

### SetExpectedOutcomeNil

`func (o *Scenario) SetExpectedOutcomeNil(b bool)`

 SetExpectedOutcomeNil sets the value for ExpectedOutcome to be an explicit nil

### UnsetExpectedOutcome
`func (o *Scenario) UnsetExpectedOutcome()`

UnsetExpectedOutcome ensures that no value is present for ExpectedOutcome, not even an explicit nil
### GetFailureMode

`func (o *Scenario) GetFailureMode() FailureMode`

GetFailureMode returns the FailureMode field if non-nil, zero value otherwise.

### GetFailureModeOk

`func (o *Scenario) GetFailureModeOk() (*FailureMode, bool)`

GetFailureModeOk returns a tuple with the FailureMode field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFailureMode

`func (o *Scenario) SetFailureMode(v FailureMode)`

SetFailureMode sets FailureMode field to given value.


### GetPerturbationKnobs

`func (o *Scenario) GetPerturbationKnobs() PerturbationKnobs`

GetPerturbationKnobs returns the PerturbationKnobs field if non-nil, zero value otherwise.

### GetPerturbationKnobsOk

`func (o *Scenario) GetPerturbationKnobsOk() (*PerturbationKnobs, bool)`

GetPerturbationKnobsOk returns a tuple with the PerturbationKnobs field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPerturbationKnobs

`func (o *Scenario) SetPerturbationKnobs(v PerturbationKnobs)`

SetPerturbationKnobs sets PerturbationKnobs field to given value.


### GetRecurrenceCount

`func (o *Scenario) GetRecurrenceCount() int32`

GetRecurrenceCount returns the RecurrenceCount field if non-nil, zero value otherwise.

### GetRecurrenceCountOk

`func (o *Scenario) GetRecurrenceCountOk() (*int32, bool)`

GetRecurrenceCountOk returns a tuple with the RecurrenceCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRecurrenceCount

`func (o *Scenario) SetRecurrenceCount(v int32)`

SetRecurrenceCount sets RecurrenceCount field to given value.


### GetRedactionClass

`func (o *Scenario) GetRedactionClass() RedactionClass`

GetRedactionClass returns the RedactionClass field if non-nil, zero value otherwise.

### GetRedactionClassOk

`func (o *Scenario) GetRedactionClassOk() (*RedactionClass, bool)`

GetRedactionClassOk returns a tuple with the RedactionClass field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRedactionClass

`func (o *Scenario) SetRedactionClass(v RedactionClass)`

SetRedactionClass sets RedactionClass field to given value.


### GetScenarioId

`func (o *Scenario) GetScenarioId() string`

GetScenarioId returns the ScenarioId field if non-nil, zero value otherwise.

### GetScenarioIdOk

`func (o *Scenario) GetScenarioIdOk() (*string, bool)`

GetScenarioIdOk returns a tuple with the ScenarioId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScenarioId

`func (o *Scenario) SetScenarioId(v string)`

SetScenarioId sets ScenarioId field to given value.


### GetScope

`func (o *Scenario) GetScope() TenantScope`

GetScope returns the Scope field if non-nil, zero value otherwise.

### GetScopeOk

`func (o *Scenario) GetScopeOk() (*TenantScope, bool)`

GetScopeOk returns a tuple with the Scope field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScope

`func (o *Scenario) SetScope(v TenantScope)`

SetScope sets Scope field to given value.


### GetSourceTraceIds

`func (o *Scenario) GetSourceTraceIds() []string`

GetSourceTraceIds returns the SourceTraceIds field if non-nil, zero value otherwise.

### GetSourceTraceIdsOk

`func (o *Scenario) GetSourceTraceIdsOk() (*[]string, bool)`

GetSourceTraceIdsOk returns a tuple with the SourceTraceIds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSourceTraceIds

`func (o *Scenario) SetSourceTraceIds(v []string)`

SetSourceTraceIds sets SourceTraceIds field to given value.


### GetTitle

`func (o *Scenario) GetTitle() string`

GetTitle returns the Title field if non-nil, zero value otherwise.

### GetTitleOk

`func (o *Scenario) GetTitleOk() (*string, bool)`

GetTitleOk returns a tuple with the Title field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTitle

`func (o *Scenario) SetTitle(v string)`

SetTitle sets Title field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
