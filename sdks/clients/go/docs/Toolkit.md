# Toolkit

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**AuthSchemes** | Pointer to **[]string** | Supported auth schemes (e.g. &#x60;OAUTH2&#x60;, &#x60;API_KEY&#x60;, &#x60;NO_AUTH&#x60;). | [optional] 
**Description** | Pointer to **NullableString** | Short description, if the catalog provides one. | [optional] 
**Name** | **string** | Human display name. | 
**NoAuth** | Pointer to **bool** | &#x60;true&#x60; when the toolkit needs no OAuth/connection to execute. | [optional] 
**Slug** | **string** | Stable slug used everywhere else (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;). | 
**ToolsCount** | Pointer to **NullableInt32** | Number of tools the toolkit exposes, if known. | [optional] 

## Methods

### NewToolkit

`func NewToolkit(name string, slug string, ) *Toolkit`

NewToolkit instantiates a new Toolkit object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewToolkitWithDefaults

`func NewToolkitWithDefaults() *Toolkit`

NewToolkitWithDefaults instantiates a new Toolkit object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAuthSchemes

`func (o *Toolkit) GetAuthSchemes() []string`

GetAuthSchemes returns the AuthSchemes field if non-nil, zero value otherwise.

### GetAuthSchemesOk

`func (o *Toolkit) GetAuthSchemesOk() (*[]string, bool)`

GetAuthSchemesOk returns a tuple with the AuthSchemes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAuthSchemes

`func (o *Toolkit) SetAuthSchemes(v []string)`

SetAuthSchemes sets AuthSchemes field to given value.

### HasAuthSchemes

`func (o *Toolkit) HasAuthSchemes() bool`

HasAuthSchemes returns a boolean if a field has been set.

### GetDescription

`func (o *Toolkit) GetDescription() string`

GetDescription returns the Description field if non-nil, zero value otherwise.

### GetDescriptionOk

`func (o *Toolkit) GetDescriptionOk() (*string, bool)`

GetDescriptionOk returns a tuple with the Description field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDescription

`func (o *Toolkit) SetDescription(v string)`

SetDescription sets Description field to given value.

### HasDescription

`func (o *Toolkit) HasDescription() bool`

HasDescription returns a boolean if a field has been set.

### SetDescriptionNil

`func (o *Toolkit) SetDescriptionNil(b bool)`

 SetDescriptionNil sets the value for Description to be an explicit nil

### UnsetDescription
`func (o *Toolkit) UnsetDescription()`

UnsetDescription ensures that no value is present for Description, not even an explicit nil
### GetName

`func (o *Toolkit) GetName() string`

GetName returns the Name field if non-nil, zero value otherwise.

### GetNameOk

`func (o *Toolkit) GetNameOk() (*string, bool)`

GetNameOk returns a tuple with the Name field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetName

`func (o *Toolkit) SetName(v string)`

SetName sets Name field to given value.


### GetNoAuth

`func (o *Toolkit) GetNoAuth() bool`

GetNoAuth returns the NoAuth field if non-nil, zero value otherwise.

### GetNoAuthOk

`func (o *Toolkit) GetNoAuthOk() (*bool, bool)`

GetNoAuthOk returns a tuple with the NoAuth field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNoAuth

`func (o *Toolkit) SetNoAuth(v bool)`

SetNoAuth sets NoAuth field to given value.

### HasNoAuth

`func (o *Toolkit) HasNoAuth() bool`

HasNoAuth returns a boolean if a field has been set.

### GetSlug

`func (o *Toolkit) GetSlug() string`

GetSlug returns the Slug field if non-nil, zero value otherwise.

### GetSlugOk

`func (o *Toolkit) GetSlugOk() (*string, bool)`

GetSlugOk returns a tuple with the Slug field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSlug

`func (o *Toolkit) SetSlug(v string)`

SetSlug sets Slug field to given value.


### GetToolsCount

`func (o *Toolkit) GetToolsCount() int32`

GetToolsCount returns the ToolsCount field if non-nil, zero value otherwise.

### GetToolsCountOk

`func (o *Toolkit) GetToolsCountOk() (*int32, bool)`

GetToolsCountOk returns a tuple with the ToolsCount field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetToolsCount

`func (o *Toolkit) SetToolsCount(v int32)`

SetToolsCount sets ToolsCount field to given value.

### HasToolsCount

`func (o *Toolkit) HasToolsCount() bool`

HasToolsCount returns a boolean if a field has been set.

### SetToolsCountNil

`func (o *Toolkit) SetToolsCountNil(b bool)`

 SetToolsCountNil sets the value for ToolsCount to be an explicit nil

### UnsetToolsCount
`func (o *Toolkit) UnsetToolsCount()`

UnsetToolsCount ensures that no value is present for ToolsCount, not even an explicit nil

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


