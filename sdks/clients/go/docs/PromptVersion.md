# PromptVersion

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Metadata** | [**PromptVersionMetadata**](PromptVersionMetadata.md) |  | 
**ProjectId** | **string** |  | 
**PromptId** | **string** |  | 
**Template** | [**PromptTemplate**](PromptTemplate.md) |  | 
**TenantId** | **string** |  | 
**VersionId** | **string** |  | 
**VersionNumber** | **int32** |  | 

## Methods

### NewPromptVersion

`func NewPromptVersion(metadata PromptVersionMetadata, projectId string, promptId string, template PromptTemplate, tenantId string, versionId string, versionNumber int32, ) *PromptVersion`

NewPromptVersion instantiates a new PromptVersion object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPromptVersionWithDefaults

`func NewPromptVersionWithDefaults() *PromptVersion`

NewPromptVersionWithDefaults instantiates a new PromptVersion object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetMetadata

`func (o *PromptVersion) GetMetadata() PromptVersionMetadata`

GetMetadata returns the Metadata field if non-nil, zero value otherwise.

### GetMetadataOk

`func (o *PromptVersion) GetMetadataOk() (*PromptVersionMetadata, bool)`

GetMetadataOk returns a tuple with the Metadata field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMetadata

`func (o *PromptVersion) SetMetadata(v PromptVersionMetadata)`

SetMetadata sets Metadata field to given value.


### GetProjectId

`func (o *PromptVersion) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *PromptVersion) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *PromptVersion) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetPromptId

`func (o *PromptVersion) GetPromptId() string`

GetPromptId returns the PromptId field if non-nil, zero value otherwise.

### GetPromptIdOk

`func (o *PromptVersion) GetPromptIdOk() (*string, bool)`

GetPromptIdOk returns a tuple with the PromptId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPromptId

`func (o *PromptVersion) SetPromptId(v string)`

SetPromptId sets PromptId field to given value.


### GetTemplate

`func (o *PromptVersion) GetTemplate() PromptTemplate`

GetTemplate returns the Template field if non-nil, zero value otherwise.

### GetTemplateOk

`func (o *PromptVersion) GetTemplateOk() (*PromptTemplate, bool)`

GetTemplateOk returns a tuple with the Template field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTemplate

`func (o *PromptVersion) SetTemplate(v PromptTemplate)`

SetTemplate sets Template field to given value.


### GetTenantId

`func (o *PromptVersion) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *PromptVersion) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *PromptVersion) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetVersionId

`func (o *PromptVersion) GetVersionId() string`

GetVersionId returns the VersionId field if non-nil, zero value otherwise.

### GetVersionIdOk

`func (o *PromptVersion) GetVersionIdOk() (*string, bool)`

GetVersionIdOk returns a tuple with the VersionId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVersionId

`func (o *PromptVersion) SetVersionId(v string)`

SetVersionId sets VersionId field to given value.


### GetVersionNumber

`func (o *PromptVersion) GetVersionNumber() int32`

GetVersionNumber returns the VersionNumber field if non-nil, zero value otherwise.

### GetVersionNumberOk

`func (o *PromptVersion) GetVersionNumberOk() (*int32, bool)`

GetVersionNumberOk returns a tuple with the VersionNumber field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetVersionNumber

`func (o *PromptVersion) SetVersionNumber(v int32)`

SetVersionNumber sets VersionNumber field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


