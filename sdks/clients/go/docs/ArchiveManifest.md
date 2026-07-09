# ArchiveManifest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**CreatedAt** | **time.Time** |  |
**Path** | **string** |  |
**ProjectId** | **string** |  |
**SpanCount** | **int32** |  |
**TenantId** | **string** |  |

## Methods

### NewArchiveManifest

`func NewArchiveManifest(createdAt time.Time, path string, projectId string, spanCount int32, tenantId string, ) *ArchiveManifest`

NewArchiveManifest instantiates a new ArchiveManifest object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewArchiveManifestWithDefaults

`func NewArchiveManifestWithDefaults() *ArchiveManifest`

NewArchiveManifestWithDefaults instantiates a new ArchiveManifest object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetCreatedAt

`func (o *ArchiveManifest) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *ArchiveManifest) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *ArchiveManifest) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetPath

`func (o *ArchiveManifest) GetPath() string`

GetPath returns the Path field if non-nil, zero value otherwise.

### GetPathOk

`func (o *ArchiveManifest) GetPathOk() (*string, bool)`

GetPathOk returns a tuple with the Path field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPath

`func (o *ArchiveManifest) SetPath(v string)`

SetPath sets Path field to given value.


### GetProjectId

`func (o *ArchiveManifest) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *ArchiveManifest) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *ArchiveManifest) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetSpanCount

`func (o *ArchiveManifest) GetSpanCount() int32`

GetSpanCount returns the SpanCount field if non-nil, zero value otherwise.

### GetSpanCountOk

`func (o *ArchiveManifest) GetSpanCountOk() (*int32, bool)`

GetSpanCountOk returns a tuple with the SpanCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSpanCount

`func (o *ArchiveManifest) SetSpanCount(v int32)`

SetSpanCount sets SpanCount field to given value.


### GetTenantId

`func (o *ArchiveManifest) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *ArchiveManifest) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *ArchiveManifest) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
