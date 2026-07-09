# DatasetVersionSnapshot

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Cases** | [**[]DatasetCase**](DatasetCase.md) |  |
**CorpusRoot** | **string** | A content-addressed Merkle root naming the exact contents of a corpus.  Serialized as its lowercase-hex SHA-256 string. |
**CreatedAt** | **time.Time** |  |
**DatasetId** | **string** |  |
**ProjectId** | **string** |  |
**TenantId** | **string** |  |
**VersionId** | **string** |  |

## Methods

### NewDatasetVersionSnapshot

`func NewDatasetVersionSnapshot(cases []DatasetCase, corpusRoot string, createdAt time.Time, datasetId string, projectId string, tenantId string, versionId string, ) *DatasetVersionSnapshot`

NewDatasetVersionSnapshot instantiates a new DatasetVersionSnapshot object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewDatasetVersionSnapshotWithDefaults

`func NewDatasetVersionSnapshotWithDefaults() *DatasetVersionSnapshot`

NewDatasetVersionSnapshotWithDefaults instantiates a new DatasetVersionSnapshot object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCases

`func (o *DatasetVersionSnapshot) GetCases() []DatasetCase`

GetCases returns the Cases field if non-nil, zero value otherwise.

### GetCasesOk

`func (o *DatasetVersionSnapshot) GetCasesOk() (*[]DatasetCase, bool)`

GetCasesOk returns a tuple with the Cases field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCases

`func (o *DatasetVersionSnapshot) SetCases(v []DatasetCase)`

SetCases sets Cases field to given value.


### GetCorpusRoot

`func (o *DatasetVersionSnapshot) GetCorpusRoot() string`

GetCorpusRoot returns the CorpusRoot field if non-nil, zero value otherwise.

### GetCorpusRootOk

`func (o *DatasetVersionSnapshot) GetCorpusRootOk() (*string, bool)`

GetCorpusRootOk returns a tuple with the CorpusRoot field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCorpusRoot

`func (o *DatasetVersionSnapshot) SetCorpusRoot(v string)`

SetCorpusRoot sets CorpusRoot field to given value.


### GetCreatedAt

`func (o *DatasetVersionSnapshot) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *DatasetVersionSnapshot) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *DatasetVersionSnapshot) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetDatasetId

`func (o *DatasetVersionSnapshot) GetDatasetId() string`

GetDatasetId returns the DatasetId field if non-nil, zero value otherwise.

### GetDatasetIdOk

`func (o *DatasetVersionSnapshot) GetDatasetIdOk() (*string, bool)`

GetDatasetIdOk returns a tuple with the DatasetId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDatasetId

`func (o *DatasetVersionSnapshot) SetDatasetId(v string)`

SetDatasetId sets DatasetId field to given value.


### GetProjectId

`func (o *DatasetVersionSnapshot) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *DatasetVersionSnapshot) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *DatasetVersionSnapshot) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetTenantId

`func (o *DatasetVersionSnapshot) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *DatasetVersionSnapshot) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *DatasetVersionSnapshot) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetVersionId

`func (o *DatasetVersionSnapshot) GetVersionId() string`

GetVersionId returns the VersionId field if non-nil, zero value otherwise.

### GetVersionIdOk

`func (o *DatasetVersionSnapshot) GetVersionIdOk() (*string, bool)`

GetVersionIdOk returns a tuple with the VersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVersionId

`func (o *DatasetVersionSnapshot) SetVersionId(v string)`

SetVersionId sets VersionId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
