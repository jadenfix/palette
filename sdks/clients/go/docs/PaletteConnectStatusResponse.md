# PaletteConnectStatusResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**FirstEvalRun** | **bool** |  |
**FirstTraceReceived** | **bool** |  |
**Ok** | **bool** |  |
**ProjectId** | **string** |  |
**Status** | [**PaletteConnectStatus**](PaletteConnectStatus.md) |  |
**TenantId** | **string** |  |
**Totals** | [**map[string]UsageTotal**](UsageTotal.md) |  |
**UsageConfigured** | **bool** |  |

## Methods

### NewPaletteConnectStatusResponse

`func NewPaletteConnectStatusResponse(firstEvalRun bool, firstTraceReceived bool, ok bool, projectId string, status PaletteConnectStatus, tenantId string, totals map[string]UsageTotal, usageConfigured bool, ) *PaletteConnectStatusResponse`

NewPaletteConnectStatusResponse instantiates a new PaletteConnectStatusResponse object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewPaletteConnectStatusResponseWithDefaults

`func NewPaletteConnectStatusResponseWithDefaults() *PaletteConnectStatusResponse`

NewPaletteConnectStatusResponseWithDefaults instantiates a new PaletteConnectStatusResponse object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetFirstEvalRun

`func (o *PaletteConnectStatusResponse) GetFirstEvalRun() bool`

GetFirstEvalRun returns the FirstEvalRun field if non-nil, zero value otherwise.

### GetFirstEvalRunOk

`func (o *PaletteConnectStatusResponse) GetFirstEvalRunOk() (*bool, bool)`

GetFirstEvalRunOk returns a tuple with the FirstEvalRun field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFirstEvalRun

`func (o *PaletteConnectStatusResponse) SetFirstEvalRun(v bool)`

SetFirstEvalRun sets FirstEvalRun field to given value.


### GetFirstTraceReceived

`func (o *PaletteConnectStatusResponse) GetFirstTraceReceived() bool`

GetFirstTraceReceived returns the FirstTraceReceived field if non-nil, zero value otherwise.

### GetFirstTraceReceivedOk

`func (o *PaletteConnectStatusResponse) GetFirstTraceReceivedOk() (*bool, bool)`

GetFirstTraceReceivedOk returns a tuple with the FirstTraceReceived field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFirstTraceReceived

`func (o *PaletteConnectStatusResponse) SetFirstTraceReceived(v bool)`

SetFirstTraceReceived sets FirstTraceReceived field to given value.


### GetOk

`func (o *PaletteConnectStatusResponse) GetOk() bool`

GetOk returns the Ok field if non-nil, zero value otherwise.

### GetOkOk

`func (o *PaletteConnectStatusResponse) GetOkOk() (*bool, bool)`

GetOkOk returns a tuple with the Ok field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOk

`func (o *PaletteConnectStatusResponse) SetOk(v bool)`

SetOk sets Ok field to given value.


### GetProjectId

`func (o *PaletteConnectStatusResponse) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *PaletteConnectStatusResponse) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *PaletteConnectStatusResponse) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetStatus

`func (o *PaletteConnectStatusResponse) GetStatus() PaletteConnectStatus`

GetStatus returns the Status field if non-nil, zero value otherwise.

### GetStatusOk

`func (o *PaletteConnectStatusResponse) GetStatusOk() (*PaletteConnectStatus, bool)`

GetStatusOk returns a tuple with the Status field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetStatus

`func (o *PaletteConnectStatusResponse) SetStatus(v PaletteConnectStatus)`

SetStatus sets Status field to given value.


### GetTenantId

`func (o *PaletteConnectStatusResponse) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *PaletteConnectStatusResponse) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *PaletteConnectStatusResponse) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.


### GetTotals

`func (o *PaletteConnectStatusResponse) GetTotals() map[string]UsageTotal`

GetTotals returns the Totals field if non-nil, zero value otherwise.

### GetTotalsOk

`func (o *PaletteConnectStatusResponse) GetTotalsOk() (*map[string]UsageTotal, bool)`

GetTotalsOk returns a tuple with the Totals field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTotals

`func (o *PaletteConnectStatusResponse) SetTotals(v map[string]UsageTotal)`

SetTotals sets Totals field to given value.


### GetUsageConfigured

`func (o *PaletteConnectStatusResponse) GetUsageConfigured() bool`

GetUsageConfigured returns the UsageConfigured field if non-nil, zero value otherwise.

### GetUsageConfiguredOk

`func (o *PaletteConnectStatusResponse) GetUsageConfiguredOk() (*bool, bool)`

GetUsageConfiguredOk returns a tuple with the UsageConfigured field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUsageConfigured

`func (o *PaletteConnectStatusResponse) SetUsageConfigured(v bool)`

SetUsageConfigured sets UsageConfigured field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
