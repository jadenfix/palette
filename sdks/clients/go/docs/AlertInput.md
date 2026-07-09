# AlertInput

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**BaselineScore** | Pointer to **NullableFloat64** |  | [optional]
**GroupKey** | **string** |  |
**Links** | [**AlertLinks**](AlertLinks.md) |  |
**Now** | **time.Time** |  |
**ProjectId** | **string** |  |
**Score** | **float64** |  |
**TenantId** | **string** |  |
**Title** | **string** |  |
**TraceId** | **string** |  |

## Methods

### NewAlertInput

`func NewAlertInput(groupKey string, links AlertLinks, now time.Time, projectId string, score float64, tenantId string, title string, traceId string, ) *AlertInput`

NewAlertInput instantiates a new AlertInput object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewAlertInputWithDefaults

`func NewAlertInputWithDefaults() *AlertInput`

NewAlertInputWithDefaults instantiates a new AlertInput object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetBaselineScore

`func (o *AlertInput) GetBaselineScore() float64`

GetBaselineScore returns the BaselineScore field if non-nil, zero value otherwise.

### GetBaselineScoreOk

`func (o *AlertInput) GetBaselineScoreOk() (*float64, bool)`

GetBaselineScoreOk returns a tuple with the BaselineScore field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetBaselineScore

`func (o *AlertInput) SetBaselineScore(v float64)`

SetBaselineScore sets BaselineScore field to given value.

### HasBaselineScore

`func (o *AlertInput) HasBaselineScore() bool`

HasBaselineScore returns a boolean if a field has been set.

### SetBaselineScoreNil

`func (o *AlertInput) SetBaselineScoreNil(b bool)`

 SetBaselineScoreNil sets the value for BaselineScore to be an explicit nil

### UnsetBaselineScore
`func (o *AlertInput) UnsetBaselineScore()`

UnsetBaselineScore ensures that no value is present for BaselineScore, not even an explicit nil
### GetGroupKey

`func (o *AlertInput) GetGroupKey() string`

GetGroupKey returns the GroupKey field if non-nil, zero value otherwise.

### GetGroupKeyOk

`func (o *AlertInput) GetGroupKeyOk() (*string, bool)`

GetGroupKeyOk returns a tuple with the GroupKey field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetGroupKey

`func (o *AlertInput) SetGroupKey(v string)`

SetGroupKey sets GroupKey field to given value.


### GetLinks

`func (o *AlertInput) GetLinks() AlertLinks`

GetLinks returns the Links field if non-nil, zero value otherwise.

### GetLinksOk

`func (o *AlertInput) GetLinksOk() (*AlertLinks, bool)`

GetLinksOk returns a tuple with the Links field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetLinks

`func (o *AlertInput) SetLinks(v AlertLinks)`

SetLinks sets Links field to given value.


### GetNow

`func (o *AlertInput) GetNow() time.Time`

GetNow returns the Now field if non-nil, zero value otherwise.

### GetNowOk

`func (o *AlertInput) GetNowOk() (*time.Time, bool)`

GetNowOk returns a tuple with the Now field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetNow

`func (o *AlertInput) SetNow(v time.Time)`

SetNow sets Now field to given value.


### GetProjectId

`func (o *AlertInput) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *AlertInput) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *AlertInput) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetScore

`func (o *AlertInput) GetScore() float64`

GetScore returns the Score field if non-nil, zero value otherwise.

### GetScoreOk

`func (o *AlertInput) GetScoreOk() (*float64, bool)`

GetScoreOk returns a tuple with the Score field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetScore

`func (o *AlertInput) SetScore(v float64)`

SetScore sets Score field to given value.


### GetTenantId

`func (o *AlertInput) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *AlertInput) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *AlertInput) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTitle

`func (o *AlertInput) GetTitle() string`

GetTitle returns the Title field if non-nil, zero value otherwise.

### GetTitleOk

`func (o *AlertInput) GetTitleOk() (*string, bool)`

GetTitleOk returns a tuple with the Title field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTitle

`func (o *AlertInput) SetTitle(v string)`

SetTitle sets Title field to given value.


### GetTraceId

`func (o *AlertInput) GetTraceId() string`

GetTraceId returns the TraceId field if non-nil, zero value otherwise.

### GetTraceIdOk

`func (o *AlertInput) GetTraceIdOk() (*string, bool)`

GetTraceIdOk returns a tuple with the TraceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTraceId

`func (o *AlertInput) SetTraceId(v string)`

SetTraceId sets TraceId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
