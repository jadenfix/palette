# ConnectorSkillsResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Skills** | **string** | Markdown document: one skill card per tool, ready to splice into an agent&#39;s system prompt. |
**Toolkit** | **string** | Toolkit the skills document covers. |

## Methods

### NewConnectorSkillsResponse

`func NewConnectorSkillsResponse(skills string, toolkit string, ) *ConnectorSkillsResponse`

NewConnectorSkillsResponse instantiates a new ConnectorSkillsResponse object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewConnectorSkillsResponseWithDefaults

`func NewConnectorSkillsResponseWithDefaults() *ConnectorSkillsResponse`

NewConnectorSkillsResponseWithDefaults instantiates a new ConnectorSkillsResponse object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetSkills

`func (o *ConnectorSkillsResponse) GetSkills() string`

GetSkills returns the Skills field if non-nil, zero value otherwise.

### GetSkillsOk

`func (o *ConnectorSkillsResponse) GetSkillsOk() (*string, bool)`

GetSkillsOk returns a tuple with the Skills field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSkills

`func (o *ConnectorSkillsResponse) SetSkills(v string)`

SetSkills sets Skills field to given value.


### GetToolkit

`func (o *ConnectorSkillsResponse) GetToolkit() string`

GetToolkit returns the Toolkit field if non-nil, zero value otherwise.

### GetToolkitOk

`func (o *ConnectorSkillsResponse) GetToolkitOk() (*string, bool)`

GetToolkitOk returns a tuple with the Toolkit field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetToolkit

`func (o *ConnectorSkillsResponse) SetToolkit(v string)`

SetToolkit sets Toolkit field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
