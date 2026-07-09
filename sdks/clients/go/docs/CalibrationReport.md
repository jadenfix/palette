# CalibrationReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BrierScore** | **float64** |  |
**CalibrationReportId** | **string** |  |
**CohenKappa** | **float64** |  |
**CohenKappaCiHigh** | Pointer to **NullableFloat64** |  | [optional]
**CohenKappaCiLow** | Pointer to **NullableFloat64** | Percentile-bootstrap 95% confidence interval for &#x60;cohen_kappa&#x60; (multinomial resampling of the confusion table, deterministic seed). Kappa over small calibration samples is high-variance; a bare point estimate invites over-reading. Absent on pre-uncertainty reports. | [optional]
**Confusion** | [**CalibrationConfusion**](CalibrationConfusion.md) |  |
**CreatedAt** | **time.Time** |  |
**DatasetId** | **string** |  |
**DatasetVersionId** | **string** |  |
**EvalReportId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**ExpectedAgreement** | **float64** |  |
**ExpectedCalibrationError** | **float64** |  |
**Items** | [**[]CalibrationItem**](CalibrationItem.md) |  |
**ObservedAgreement** | **float64** |  |
**ObservedAgreementCiHigh** | Pointer to **NullableFloat64** |  | [optional]
**ObservedAgreementCiLow** | Pointer to **NullableFloat64** | Wilson 95% confidence interval for &#x60;observed_agreement&#x60; — the honest width of an agreement estimate over a (typically small) human-labelled sample. Absent on reports persisted before uncertainty was reported. | [optional]
**Policy** | [**CalibrationPolicy**](CalibrationPolicy.md) |  |
**ProjectId** | **string** |  |
**ReliabilityBins** | [**[]ReliabilityBin**](ReliabilityBin.md) |  |
**SampleCount** | **int32** |  |
**TenantId** | **string** |  |

## Methods

### NewCalibrationReport

`func NewCalibrationReport(brierScore float64, calibrationReportId string, cohenKappa float64, confusion CalibrationConfusion, createdAt time.Time, datasetId string, datasetVersionId string, evalReportId string, evaluatorVersionId string, expectedAgreement float64, expectedCalibrationError float64, items []CalibrationItem, observedAgreement float64, policy CalibrationPolicy, projectId string, reliabilityBins []ReliabilityBin, sampleCount int32, tenantId string, ) *CalibrationReport`

NewCalibrationReport instantiates a new CalibrationReport object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewCalibrationReportWithDefaults

`func NewCalibrationReportWithDefaults() *CalibrationReport`

NewCalibrationReportWithDefaults instantiates a new CalibrationReport object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBrierScore

`func (o *CalibrationReport) GetBrierScore() float64`

GetBrierScore returns the BrierScore field if non-nil, zero value otherwise.

### GetBrierScoreOk

`func (o *CalibrationReport) GetBrierScoreOk() (*float64, bool)`

GetBrierScoreOk returns a tuple with the BrierScore field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBrierScore

`func (o *CalibrationReport) SetBrierScore(v float64)`

SetBrierScore sets BrierScore field to given value.


### GetCalibrationReportId

`func (o *CalibrationReport) GetCalibrationReportId() string`

GetCalibrationReportId returns the CalibrationReportId field if non-nil, zero value otherwise.

### GetCalibrationReportIdOk

`func (o *CalibrationReport) GetCalibrationReportIdOk() (*string, bool)`

GetCalibrationReportIdOk returns a tuple with the CalibrationReportId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCalibrationReportId

`func (o *CalibrationReport) SetCalibrationReportId(v string)`

SetCalibrationReportId sets CalibrationReportId field to given value.


### GetCohenKappa

`func (o *CalibrationReport) GetCohenKappa() float64`

GetCohenKappa returns the CohenKappa field if non-nil, zero value otherwise.

### GetCohenKappaOk

`func (o *CalibrationReport) GetCohenKappaOk() (*float64, bool)`

GetCohenKappaOk returns a tuple with the CohenKappa field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCohenKappa

`func (o *CalibrationReport) SetCohenKappa(v float64)`

SetCohenKappa sets CohenKappa field to given value.


### GetCohenKappaCiHigh

`func (o *CalibrationReport) GetCohenKappaCiHigh() float64`

GetCohenKappaCiHigh returns the CohenKappaCiHigh field if non-nil, zero value otherwise.

### GetCohenKappaCiHighOk

`func (o *CalibrationReport) GetCohenKappaCiHighOk() (*float64, bool)`

GetCohenKappaCiHighOk returns a tuple with the CohenKappaCiHigh field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCohenKappaCiHigh

`func (o *CalibrationReport) SetCohenKappaCiHigh(v float64)`

SetCohenKappaCiHigh sets CohenKappaCiHigh field to given value.

### HasCohenKappaCiHigh

`func (o *CalibrationReport) HasCohenKappaCiHigh() bool`

HasCohenKappaCiHigh returns a boolean if a field has been set.

### SetCohenKappaCiHighNil

`func (o *CalibrationReport) SetCohenKappaCiHighNil(b bool)`

 SetCohenKappaCiHighNil sets the value for CohenKappaCiHigh to be an explicit nil

### UnsetCohenKappaCiHigh
`func (o *CalibrationReport) UnsetCohenKappaCiHigh()`

UnsetCohenKappaCiHigh ensures that no value is present for CohenKappaCiHigh, not even an explicit nil
### GetCohenKappaCiLow

`func (o *CalibrationReport) GetCohenKappaCiLow() float64`

GetCohenKappaCiLow returns the CohenKappaCiLow field if non-nil, zero value otherwise.

### GetCohenKappaCiLowOk

`func (o *CalibrationReport) GetCohenKappaCiLowOk() (*float64, bool)`

GetCohenKappaCiLowOk returns a tuple with the CohenKappaCiLow field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCohenKappaCiLow

`func (o *CalibrationReport) SetCohenKappaCiLow(v float64)`

SetCohenKappaCiLow sets CohenKappaCiLow field to given value.

### HasCohenKappaCiLow

`func (o *CalibrationReport) HasCohenKappaCiLow() bool`

HasCohenKappaCiLow returns a boolean if a field has been set.

### SetCohenKappaCiLowNil

`func (o *CalibrationReport) SetCohenKappaCiLowNil(b bool)`

 SetCohenKappaCiLowNil sets the value for CohenKappaCiLow to be an explicit nil

### UnsetCohenKappaCiLow
`func (o *CalibrationReport) UnsetCohenKappaCiLow()`

UnsetCohenKappaCiLow ensures that no value is present for CohenKappaCiLow, not even an explicit nil
### GetConfusion

`func (o *CalibrationReport) GetConfusion() CalibrationConfusion`

GetConfusion returns the Confusion field if non-nil, zero value otherwise.

### GetConfusionOk

`func (o *CalibrationReport) GetConfusionOk() (*CalibrationConfusion, bool)`

GetConfusionOk returns a tuple with the Confusion field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetConfusion

`func (o *CalibrationReport) SetConfusion(v CalibrationConfusion)`

SetConfusion sets Confusion field to given value.


### GetCreatedAt

`func (o *CalibrationReport) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *CalibrationReport) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *CalibrationReport) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetId

`func (o *CalibrationReport) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *CalibrationReport) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *CalibrationReport) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.


### GetDatasetVersionId

`func (o *CalibrationReport) GetDatasetVersionId() string`

GetDatasetVersionId returns the DatasetVersionId field if non-nil, zero value otherwise.

### GetDatasetVersionIdOk

`func (o *CalibrationReport) GetDatasetVersionIdOk() (*string, bool)`

GetDatasetVersionIdOk returns a tuple with the DatasetVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetVersionId

`func (o *CalibrationReport) SetDatasetVersionId(v string)`

SetDatasetVersionId sets DatasetVersionId field to given value.


### GetEvalReportId

`func (o *CalibrationReport) GetEvalReportId() string`

GetEvalReportId returns the EvalReportId field if non-nil, zero value otherwise.

### GetEvalReportIdOk

`func (o *CalibrationReport) GetEvalReportIdOk() (*string, bool)`

GetEvalReportIdOk returns a tuple with the EvalReportId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvalReportId

`func (o *CalibrationReport) SetEvalReportId(v string)`

SetEvalReportId sets EvalReportId field to given value.


### GetEvaluatorVersionId

`func (o *CalibrationReport) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *CalibrationReport) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *CalibrationReport) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetExpectedAgreement

`func (o *CalibrationReport) GetExpectedAgreement() float64`

GetExpectedAgreement returns the ExpectedAgreement field if non-nil, zero value otherwise.

### GetExpectedAgreementOk

`func (o *CalibrationReport) GetExpectedAgreementOk() (*float64, bool)`

GetExpectedAgreementOk returns a tuple with the ExpectedAgreement field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExpectedAgreement

`func (o *CalibrationReport) SetExpectedAgreement(v float64)`

SetExpectedAgreement sets ExpectedAgreement field to given value.


### GetExpectedCalibrationError

`func (o *CalibrationReport) GetExpectedCalibrationError() float64`

GetExpectedCalibrationError returns the ExpectedCalibrationError field if non-nil, zero value otherwise.

### GetExpectedCalibrationErrorOk

`func (o *CalibrationReport) GetExpectedCalibrationErrorOk() (*float64, bool)`

GetExpectedCalibrationErrorOk returns a tuple with the ExpectedCalibrationError field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetExpectedCalibrationError

`func (o *CalibrationReport) SetExpectedCalibrationError(v float64)`

SetExpectedCalibrationError sets ExpectedCalibrationError field to given value.


### GetItems

`func (o *CalibrationReport) GetItems() []CalibrationItem`

GetItems returns the Items field if non-nil, zero value otherwise.

### GetItemsOk

`func (o *CalibrationReport) GetItemsOk() (*[]CalibrationItem, bool)`

GetItemsOk returns a tuple with the Items field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetItems

`func (o *CalibrationReport) SetItems(v []CalibrationItem)`

SetItems sets Items field to given value.


### GetObservedAgreement

`func (o *CalibrationReport) GetObservedAgreement() float64`

GetObservedAgreement returns the ObservedAgreement field if non-nil, zero value otherwise.

### GetObservedAgreementOk

`func (o *CalibrationReport) GetObservedAgreementOk() (*float64, bool)`

GetObservedAgreementOk returns a tuple with the ObservedAgreement field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetObservedAgreement

`func (o *CalibrationReport) SetObservedAgreement(v float64)`

SetObservedAgreement sets ObservedAgreement field to given value.


### GetObservedAgreementCiHigh

`func (o *CalibrationReport) GetObservedAgreementCiHigh() float64`

GetObservedAgreementCiHigh returns the ObservedAgreementCiHigh field if non-nil, zero value otherwise.

### GetObservedAgreementCiHighOk

`func (o *CalibrationReport) GetObservedAgreementCiHighOk() (*float64, bool)`

GetObservedAgreementCiHighOk returns a tuple with the ObservedAgreementCiHigh field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetObservedAgreementCiHigh

`func (o *CalibrationReport) SetObservedAgreementCiHigh(v float64)`

SetObservedAgreementCiHigh sets ObservedAgreementCiHigh field to given value.

### HasObservedAgreementCiHigh

`func (o *CalibrationReport) HasObservedAgreementCiHigh() bool`

HasObservedAgreementCiHigh returns a boolean if a field has been set.

### SetObservedAgreementCiHighNil

`func (o *CalibrationReport) SetObservedAgreementCiHighNil(b bool)`

 SetObservedAgreementCiHighNil sets the value for ObservedAgreementCiHigh to be an explicit nil

### UnsetObservedAgreementCiHigh
`func (o *CalibrationReport) UnsetObservedAgreementCiHigh()`

UnsetObservedAgreementCiHigh ensures that no value is present for ObservedAgreementCiHigh, not even an explicit nil
### GetObservedAgreementCiLow

`func (o *CalibrationReport) GetObservedAgreementCiLow() float64`

GetObservedAgreementCiLow returns the ObservedAgreementCiLow field if non-nil, zero value otherwise.

### GetObservedAgreementCiLowOk

`func (o *CalibrationReport) GetObservedAgreementCiLowOk() (*float64, bool)`

GetObservedAgreementCiLowOk returns a tuple with the ObservedAgreementCiLow field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetObservedAgreementCiLow

`func (o *CalibrationReport) SetObservedAgreementCiLow(v float64)`

SetObservedAgreementCiLow sets ObservedAgreementCiLow field to given value.

### HasObservedAgreementCiLow

`func (o *CalibrationReport) HasObservedAgreementCiLow() bool`

HasObservedAgreementCiLow returns a boolean if a field has been set.

### SetObservedAgreementCiLowNil

`func (o *CalibrationReport) SetObservedAgreementCiLowNil(b bool)`

 SetObservedAgreementCiLowNil sets the value for ObservedAgreementCiLow to be an explicit nil

### UnsetObservedAgreementCiLow
`func (o *CalibrationReport) UnsetObservedAgreementCiLow()`

UnsetObservedAgreementCiLow ensures that no value is present for ObservedAgreementCiLow, not even an explicit nil
### GetPolicy

`func (o *CalibrationReport) GetPolicy() CalibrationPolicy`

GetPolicy returns the Policy field if non-nil, zero value otherwise.

### GetPolicyOk

`func (o *CalibrationReport) GetPolicyOk() (*CalibrationPolicy, bool)`

GetPolicyOk returns a tuple with the Policy field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPolicy

`func (o *CalibrationReport) SetPolicy(v CalibrationPolicy)`

SetPolicy sets Policy field to given value.


### GetProjectId

`func (o *CalibrationReport) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *CalibrationReport) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *CalibrationReport) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetReliabilityBins

`func (o *CalibrationReport) GetReliabilityBins() []ReliabilityBin`

GetReliabilityBins returns the ReliabilityBins field if non-nil, zero value otherwise.

### GetReliabilityBinsOk

`func (o *CalibrationReport) GetReliabilityBinsOk() (*[]ReliabilityBin, bool)`

GetReliabilityBinsOk returns a tuple with the ReliabilityBins field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReliabilityBins

`func (o *CalibrationReport) SetReliabilityBins(v []ReliabilityBin)`

SetReliabilityBins sets ReliabilityBins field to given value.


### GetSampleCount

`func (o *CalibrationReport) GetSampleCount() int32`

GetSampleCount returns the SampleCount field if non-nil, zero value otherwise.

### GetSampleCountOk

`func (o *CalibrationReport) GetSampleCountOk() (*int32, bool)`

GetSampleCountOk returns a tuple with the SampleCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSampleCount

`func (o *CalibrationReport) SetSampleCount(v int32)`

SetSampleCount sets SampleCount field to given value.


### GetTenantId

`func (o *CalibrationReport) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *CalibrationReport) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *CalibrationReport) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
