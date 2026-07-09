# DatasetEvalReport

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AggregateScore** | **float64** |  |
**CreatedAt** | **time.Time** |  |
**DatasetId** | **string** |  |
**DatasetVersionId** | **string** |  |
**EvaluatorVersionId** | **string** |  |
**ProjectId** | **string** |  |
**ReportId** | **string** |  |
**ResultCount** | **int32** |  |
**Results** | [**[]EvalResult**](EvalResult.md) |  |
**TenantId** | **string** |  |

## Methods

### NewDatasetEvalReport

`func NewDatasetEvalReport(aggregateScore float64, createdAt time.Time, datasetId string, datasetVersionId string, evaluatorVersionId string, projectId string, reportId string, resultCount int32, results []EvalResult, tenantId string, ) *DatasetEvalReport`

NewDatasetEvalReport instantiates a new DatasetEvalReport object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewDatasetEvalReportWithDefaults

`func NewDatasetEvalReportWithDefaults() *DatasetEvalReport`

NewDatasetEvalReportWithDefaults instantiates a new DatasetEvalReport object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAggregateScore

`func (o *DatasetEvalReport) GetAggregateScore() float64`

GetAggregateScore returns the AggregateScore field if non-nil, zero value otherwise.

### GetAggregateScoreOk

`func (o *DatasetEvalReport) GetAggregateScoreOk() (*float64, bool)`

GetAggregateScoreOk returns a tuple with the AggregateScore field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAggregateScore

`func (o *DatasetEvalReport) SetAggregateScore(v float64)`

SetAggregateScore sets AggregateScore field to given value.


### GetCreatedAt

`func (o *DatasetEvalReport) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *DatasetEvalReport) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *DatasetEvalReport) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetId

`func (o *DatasetEvalReport) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *DatasetEvalReport) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *DatasetEvalReport) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.


### GetDatasetVersionId

`func (o *DatasetEvalReport) GetDatasetVersionId() string`

GetDatasetVersionId returns the DatasetVersionId field if non-nil, zero value otherwise.

### GetDatasetVersionIdOk

`func (o *DatasetEvalReport) GetDatasetVersionIdOk() (*string, bool)`

GetDatasetVersionIdOk returns a tuple with the DatasetVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetVersionId

`func (o *DatasetEvalReport) SetDatasetVersionId(v string)`

SetDatasetVersionId sets DatasetVersionId field to given value.


### GetEvaluatorVersionId

`func (o *DatasetEvalReport) GetEvaluatorVersionId() string`

GetEvaluatorVersionId returns the EvaluatorVersionId field if non-nil, zero value otherwise.

### GetEvaluatorVersionIdOk

`func (o *DatasetEvalReport) GetEvaluatorVersionIdOk() (*string, bool)`

GetEvaluatorVersionIdOk returns a tuple with the EvaluatorVersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEvaluatorVersionId

`func (o *DatasetEvalReport) SetEvaluatorVersionId(v string)`

SetEvaluatorVersionId sets EvaluatorVersionId field to given value.


### GetProjectId

`func (o *DatasetEvalReport) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *DatasetEvalReport) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *DatasetEvalReport) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetReportId

`func (o *DatasetEvalReport) GetReportId() string`

GetReportId returns the ReportId field if non-nil, zero value otherwise.

### GetReportIdOk

`func (o *DatasetEvalReport) GetReportIdOk() (*string, bool)`

GetReportIdOk returns a tuple with the ReportId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReportId

`func (o *DatasetEvalReport) SetReportId(v string)`

SetReportId sets ReportId field to given value.


### GetResultCount

`func (o *DatasetEvalReport) GetResultCount() int32`

GetResultCount returns the ResultCount field if non-nil, zero value otherwise.

### GetResultCountOk

`func (o *DatasetEvalReport) GetResultCountOk() (*int32, bool)`

GetResultCountOk returns a tuple with the ResultCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResultCount

`func (o *DatasetEvalReport) SetResultCount(v int32)`

SetResultCount sets ResultCount field to given value.


### GetResults

`func (o *DatasetEvalReport) GetResults() []EvalResult`

GetResults returns the Results field if non-nil, zero value otherwise.

### GetResultsOk

`func (o *DatasetEvalReport) GetResultsOk() (*[]EvalResult, bool)`

GetResultsOk returns a tuple with the Results field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResults

`func (o *DatasetEvalReport) SetResults(v []EvalResult)`

SetResults sets Results field to given value.


### GetTenantId

`func (o *DatasetEvalReport) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *DatasetEvalReport) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *DatasetEvalReport) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
