# UsageSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ProjectId** | **string** |  |
**TenantId** | **string** |  |
**Totals** | [**map[string]UsageTotal**](UsageTotal.md) |  |

## Methods

### NewUsageSummary

`func NewUsageSummary(projectId string, tenantId string, totals map[string]UsageTotal, ) *UsageSummary`

NewUsageSummary instantiates a new UsageSummary object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewUsageSummaryWithDefaults

`func NewUsageSummaryWithDefaults() *UsageSummary`

NewUsageSummaryWithDefaults instantiates a new UsageSummary object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetProjectId

`func (o *UsageSummary) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *UsageSummary) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *UsageSummary) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetTenantId

`func (o *UsageSummary) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *UsageSummary) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *UsageSummary) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTotals

`func (o *UsageSummary) GetTotals() map[string]UsageTotal`

GetTotals returns the Totals field if non-nil, zero value otherwise.

### GetTotalsOk

`func (o *UsageSummary) GetTotalsOk() (*map[string]UsageTotal, bool)`

GetTotalsOk returns a tuple with the Totals field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTotals

`func (o *UsageSummary) SetTotals(v map[string]UsageTotal)`

SetTotals sets Totals field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
