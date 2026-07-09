# AlertPolicy

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**DedupeWindowSeconds** | **int64** |  |
**EndpointUrl** | **string** |  |
**FireWhenScoreAtOrBelow** | **float64** |  |
**MaintenanceWindows** | [**[]MaintenanceWindow**](MaintenanceWindow.md) |  |
**PolicyId** | **string** |  |
**Severity** | [**AlertSeverity**](AlertSeverity.md) |  |
**SigningSecret** | **string** |  |

## Methods

### NewAlertPolicy

`func NewAlertPolicy(dedupeWindowSeconds int64, endpointUrl string, fireWhenScoreAtOrBelow float64, maintenanceWindows []MaintenanceWindow, policyId string, severity AlertSeverity, signingSecret string, ) *AlertPolicy`

NewAlertPolicy instantiates a new AlertPolicy object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewAlertPolicyWithDefaults

`func NewAlertPolicyWithDefaults() *AlertPolicy`

NewAlertPolicyWithDefaults instantiates a new AlertPolicy object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetDedupeWindowSeconds

`func (o *AlertPolicy) GetDedupeWindowSeconds() int64`

GetDedupeWindowSeconds returns the DedupeWindowSeconds field if non-nil, zero value otherwise.

### GetDedupeWindowSecondsOk

`func (o *AlertPolicy) GetDedupeWindowSecondsOk() (*int64, bool)`

GetDedupeWindowSecondsOk returns a tuple with the DedupeWindowSeconds field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetDedupeWindowSeconds

`func (o *AlertPolicy) SetDedupeWindowSeconds(v int64)`

SetDedupeWindowSeconds sets DedupeWindowSeconds field to given value.


### GetEndpointUrl

`func (o *AlertPolicy) GetEndpointUrl() string`

GetEndpointUrl returns the EndpointUrl field if non-nil, zero value otherwise.

### GetEndpointUrlOk

`func (o *AlertPolicy) GetEndpointUrlOk() (*string, bool)`

GetEndpointUrlOk returns a tuple with the EndpointUrl field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEndpointUrl

`func (o *AlertPolicy) SetEndpointUrl(v string)`

SetEndpointUrl sets EndpointUrl field to given value.


### GetFireWhenScoreAtOrBelow

`func (o *AlertPolicy) GetFireWhenScoreAtOrBelow() float64`

GetFireWhenScoreAtOrBelow returns the FireWhenScoreAtOrBelow field if non-nil, zero value otherwise.

### GetFireWhenScoreAtOrBelowOk

`func (o *AlertPolicy) GetFireWhenScoreAtOrBelowOk() (*float64, bool)`

GetFireWhenScoreAtOrBelowOk returns a tuple with the FireWhenScoreAtOrBelow field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetFireWhenScoreAtOrBelow

`func (o *AlertPolicy) SetFireWhenScoreAtOrBelow(v float64)`

SetFireWhenScoreAtOrBelow sets FireWhenScoreAtOrBelow field to given value.


### GetMaintenanceWindows

`func (o *AlertPolicy) GetMaintenanceWindows() []MaintenanceWindow`

GetMaintenanceWindows returns the MaintenanceWindows field if non-nil, zero value otherwise.

### GetMaintenanceWindowsOk

`func (o *AlertPolicy) GetMaintenanceWindowsOk() (*[]MaintenanceWindow, bool)`

GetMaintenanceWindowsOk returns a tuple with the MaintenanceWindows field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMaintenanceWindows

`func (o *AlertPolicy) SetMaintenanceWindows(v []MaintenanceWindow)`

SetMaintenanceWindows sets MaintenanceWindows field to given value.


### GetPolicyId

`func (o *AlertPolicy) GetPolicyId() string`

GetPolicyId returns the PolicyId field if non-nil, zero value otherwise.

### GetPolicyIdOk

`func (o *AlertPolicy) GetPolicyIdOk() (*string, bool)`

GetPolicyIdOk returns a tuple with the PolicyId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetPolicyId

`func (o *AlertPolicy) SetPolicyId(v string)`

SetPolicyId sets PolicyId field to given value.


### GetSeverity

`func (o *AlertPolicy) GetSeverity() AlertSeverity`

GetSeverity returns the Severity field if non-nil, zero value otherwise.

### GetSeverityOk

`func (o *AlertPolicy) GetSeverityOk() (*AlertSeverity, bool)`

GetSeverityOk returns a tuple with the Severity field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSeverity

`func (o *AlertPolicy) SetSeverity(v AlertSeverity)`

SetSeverity sets Severity field to given value.


### GetSigningSecret

`func (o *AlertPolicy) GetSigningSecret() string`

GetSigningSecret returns the SigningSecret field if non-nil, zero value otherwise.

### GetSigningSecretOk

`func (o *AlertPolicy) GetSigningSecretOk() (*string, bool)`

GetSigningSecretOk returns a tuple with the SigningSecret field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSigningSecret

`func (o *AlertPolicy) SetSigningSecret(v string)`

SetSigningSecret sets SigningSecret field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
