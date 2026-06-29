# ConnectorTool

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Description** | Pointer to **NullableString** | What the tool does. | [optional] 
**InputSchema** | Pointer to **map[string]interface{}** | JSON Schema of the tool&#39;s &#x60;arguments&#x60;, verbatim from Composio. The agent loop uses this to construct valid calls; [&#x60;crate::skill&#x60;] renders it. | [optional] 
**Name** | **string** | Human display name. | 
**NoAuth** | Pointer to **bool** | &#x60;true&#x60; when the tool executes without a connected account. | [optional] 
**Slug** | **string** | Tool slug passed to [&#x60;ComposioClient::execute&#x60;] (e.g. &#x60;GITHUB_CREATE_AN_ISSUE&#x60;). | 
**Tags** | Pointer to **[]string** | Free-form tags Composio assigns (categories, importance, …). | [optional] 
**Toolkit** | Pointer to **NullableString** | Owning toolkit slug (e.g. &#x60;github&#x60;), when known. | [optional] 

## Methods

### NewConnectorTool

`func NewConnectorTool(name string, slug string, ) *ConnectorTool`

NewConnectorTool instantiates a new ConnectorTool object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewConnectorToolWithDefaults

`func NewConnectorToolWithDefaults() *ConnectorTool`

NewConnectorToolWithDefaults instantiates a new ConnectorTool object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDescription

`func (o *ConnectorTool) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *ConnectorTool) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *ConnectorTool) SetDescription(v string)`

SetDescription sets Description field to given value.

### HasDescription

`func (o *ConnectorTool) HasDescription() bool`

HasDescription returns a boolean if a field has been set.

### SetDescriptionNil

`func (o *ConnectorTool) SetDescriptionNil(b bool)`

 SetDescriptionNil sets the value for Description to be an explicit nil

### UnsetDescription
`func (o *ConnectorTool) UnsetDescription()`

UnsetDescription ensures that no value is present for Description, not even an explicit nil
### GetInputSchema

`func (o *ConnectorTool) GetInputSchema() map[string]interface{}`

GetInputSchema returns the InputSchema field if non-nil, zero value otherwise.

### GetInputSchemaOk

`func (o *ConnectorTool) GetInputSchemaOk() (*map[string]interface{}, bool)`

GetInputSchemaOk returns a tuple with the InputSchema field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetInputSchema

`func (o *ConnectorTool) SetInputSchema(v map[string]interface{})`

SetInputSchema sets InputSchema field to given value.

### HasInputSchema

`func (o *ConnectorTool) HasInputSchema() bool`

HasInputSchema returns a boolean if a field has been set.

### SetInputSchemaNil

`func (o *ConnectorTool) SetInputSchemaNil(b bool)`

 SetInputSchemaNil sets the value for InputSchema to be an explicit nil

### UnsetInputSchema
`func (o *ConnectorTool) UnsetInputSchema()`

UnsetInputSchema ensures that no value is present for InputSchema, not even an explicit nil
### GetName

`func (o *ConnectorTool) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *ConnectorTool) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *ConnectorTool) SetName(v string)`

SetName sets Name field to given value.


### GetNoAuth

`func (o *ConnectorTool) GetNoAuth() bool`

GetNoAuth returns the NoAuth field if non-nil, zero value otherwise.

### GetNoAuthOk

`func (o *ConnectorTool) GetNoAuthOk() (*bool, bool)`

GetNoAuthOk returns a tuple with the NoAuth field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNoAuth

`func (o *ConnectorTool) SetNoAuth(v bool)`

SetNoAuth sets NoAuth field to given value.

### HasNoAuth

`func (o *ConnectorTool) HasNoAuth() bool`

HasNoAuth returns a boolean if a field has been set.

### GetSlug

`func (o *ConnectorTool) GetSlug() string`

GetSlug returns the Slug field if non-nil, zero value otherwise.

### GetSlugOk

`func (o *ConnectorTool) GetSlugOk() (*string, bool)`

GetSlugOk returns a tuple with the Slug field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSlug

`func (o *ConnectorTool) SetSlug(v string)`

SetSlug sets Slug field to given value.


### GetTags

`func (o *ConnectorTool) GetTags() []string`

GetTags returns the Tags field if non-nil, zero value otherwise.

### GetTagsOk

`func (o *ConnectorTool) GetTagsOk() (*[]string, bool)`

GetTagsOk returns a tuple with the Tags field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTags

`func (o *ConnectorTool) SetTags(v []string)`

SetTags sets Tags field to given value.

### HasTags

`func (o *ConnectorTool) HasTags() bool`

HasTags returns a boolean if a field has been set.

### GetToolkit

`func (o *ConnectorTool) GetToolkit() string`

GetToolkit returns the Toolkit field if non-nil, zero value otherwise.

### GetToolkitOk

`func (o *ConnectorTool) GetToolkitOk() (*string, bool)`

GetToolkitOk returns a tuple with the Toolkit field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetToolkit

`func (o *ConnectorTool) SetToolkit(v string)`

SetToolkit sets Toolkit field to given value.

### HasToolkit

`func (o *ConnectorTool) HasToolkit() bool`

HasToolkit returns a boolean if a field has been set.

### SetToolkitNil

`func (o *ConnectorTool) SetToolkitNil(b bool)`

 SetToolkitNil sets the value for Toolkit to be an explicit nil

### UnsetToolkit
`func (o *ConnectorTool) UnsetToolkit()`

UnsetToolkit ensures that no value is present for Toolkit, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


