# AuditEvent

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Action** | [**AuditAction**](AuditAction.md) |  |
**ActorApiKeyId** | Pointer to **string** |  | [optional]
**Attributes** | **interface{}** |  |
**AuditEventId** | **string** |  |
**CreatedAt** | **time.Time** |  |
**EnvironmentId** | Pointer to **string** |  | [optional]
**Outcome** | [**AuditOutcome**](AuditOutcome.md) |  |
**ProjectId** | **string** |  |
**Reason** | Pointer to **NullableString** |  | [optional]
**ResourceId** | **string** |  |
**ResourceType** | **string** |  |
**TenantId** | **string** |  |

## Methods

### NewAuditEvent

`func NewAuditEvent(action AuditAction, attributes interface{}, auditEventId string, createdAt time.Time, outcome AuditOutcome, projectId string, resourceId string, resourceType string, tenantId string, ) *AuditEvent`

NewAuditEvent instantiates a new AuditEvent object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewAuditEventWithDefaults

`func NewAuditEventWithDefaults() *AuditEvent`

NewAuditEventWithDefaults instantiates a new AuditEvent object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetAction

`func (o *AuditEvent) GetAction() AuditAction`

GetAction returns the Action field if non-nil, zero value otherwise.

### GetActionOk

`func (o *AuditEvent) GetActionOk() (*AuditAction, bool)`

GetActionOk returns a tuple with the Action field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAction

`func (o *AuditEvent) SetAction(v AuditAction)`

SetAction sets Action field to given value.


### GetActorApiKeyId

`func (o *AuditEvent) GetActorApiKeyId() string`

GetActorApiKeyId returns the ActorApiKeyId field if non-nil, zero value otherwise.

### GetActorApiKeyIdOk

`func (o *AuditEvent) GetActorApiKeyIdOk() (*string, bool)`

GetActorApiKeyIdOk returns a tuple with the ActorApiKeyId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetActorApiKeyId

`func (o *AuditEvent) SetActorApiKeyId(v string)`

SetActorApiKeyId sets ActorApiKeyId field to given value.

### HasActorApiKeyId

`func (o *AuditEvent) HasActorApiKeyId() bool`

HasActorApiKeyId returns a boolean if a field has been set.

### GetAttributes

`func (o *AuditEvent) GetAttributes() interface{}`

GetAttributes returns the Attributes field if non-nil, zero value otherwise.

### GetAttributesOk

`func (o *AuditEvent) GetAttributesOk() (*interface{}, bool)`

GetAttributesOk returns a tuple with the Attributes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAttributes

`func (o *AuditEvent) SetAttributes(v interface{})`

SetAttributes sets Attributes field to given value.


### SetAttributesNil

`func (o *AuditEvent) SetAttributesNil(b bool)`

 SetAttributesNil sets the value for Attributes to be an explicit nil

### UnsetAttributes
`func (o *AuditEvent) UnsetAttributes()`

UnsetAttributes ensures that no value is present for Attributes, not even an explicit nil
### GetAuditEventId

`func (o *AuditEvent) GetAuditEventId() string`

GetAuditEventId returns the AuditEventId field if non-nil, zero value otherwise.

### GetAuditEventIdOk

`func (o *AuditEvent) GetAuditEventIdOk() (*string, bool)`

GetAuditEventIdOk returns a tuple with the AuditEventId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetAuditEventId

`func (o *AuditEvent) SetAuditEventId(v string)`

SetAuditEventId sets AuditEventId field to given value.


### GetCreatedAt

`func (o *AuditEvent) GetCreatedAt() time.Time`

GetCreatedAt returns the CreatedAt field if non-nil, zero value otherwise.

### GetCreatedAtOk

`func (o *AuditEvent) GetCreatedAtOk() (*time.Time, bool)`

GetCreatedAtOk returns a tuple with the CreatedAt field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetCreatedAt

`func (o *AuditEvent) SetCreatedAt(v time.Time)`

SetCreatedAt sets CreatedAt field to given value.


### GetEnvironmentId

`func (o *AuditEvent) GetEnvironmentId() string`

GetEnvironmentId returns the EnvironmentId field if non-nil, zero value otherwise.

### GetEnvironmentIdOk

`func (o *AuditEvent) GetEnvironmentIdOk() (*string, bool)`

GetEnvironmentIdOk returns a tuple with the EnvironmentId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetEnvironmentId

`func (o *AuditEvent) SetEnvironmentId(v string)`

SetEnvironmentId sets EnvironmentId field to given value.

### HasEnvironmentId

`func (o *AuditEvent) HasEnvironmentId() bool`

HasEnvironmentId returns a boolean if a field has been set.

### GetOutcome

`func (o *AuditEvent) GetOutcome() AuditOutcome`

GetOutcome returns the Outcome field if non-nil, zero value otherwise.

### GetOutcomeOk

`func (o *AuditEvent) GetOutcomeOk() (*AuditOutcome, bool)`

GetOutcomeOk returns a tuple with the Outcome field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetOutcome

`func (o *AuditEvent) SetOutcome(v AuditOutcome)`

SetOutcome sets Outcome field to given value.


### GetProjectId

`func (o *AuditEvent) GetProjectId() string`

GetProjectId returns the ProjectId field if non-nil, zero value otherwise.

### GetProjectIdOk

`func (o *AuditEvent) GetProjectIdOk() (*string, bool)`

GetProjectIdOk returns a tuple with the ProjectId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetProjectId

`func (o *AuditEvent) SetProjectId(v string)`

SetProjectId sets ProjectId field to given value.


### GetReason

`func (o *AuditEvent) GetReason() string`

GetReason returns the Reason field if non-nil, zero value otherwise.

### GetReasonOk

`func (o *AuditEvent) GetReasonOk() (*string, bool)`

GetReasonOk returns a tuple with the Reason field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetReason

`func (o *AuditEvent) SetReason(v string)`

SetReason sets Reason field to given value.

### HasReason

`func (o *AuditEvent) HasReason() bool`

HasReason returns a boolean if a field has been set.

### SetReasonNil

`func (o *AuditEvent) SetReasonNil(b bool)`

 SetReasonNil sets the value for Reason to be an explicit nil

### UnsetReason
`func (o *AuditEvent) UnsetReason()`

UnsetReason ensures that no value is present for Reason, not even an explicit nil
### GetResourceId

`func (o *AuditEvent) GetResourceId() string`

GetResourceId returns the ResourceId field if non-nil, zero value otherwise.

### GetResourceIdOk

`func (o *AuditEvent) GetResourceIdOk() (*string, bool)`

GetResourceIdOk returns a tuple with the ResourceId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResourceId

`func (o *AuditEvent) SetResourceId(v string)`

SetResourceId sets ResourceId field to given value.


### GetResourceType

`func (o *AuditEvent) GetResourceType() string`

GetResourceType returns the ResourceType field if non-nil, zero value otherwise.

### GetResourceTypeOk

`func (o *AuditEvent) GetResourceTypeOk() (*string, bool)`

GetResourceTypeOk returns a tuple with the ResourceType field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetResourceType

`func (o *AuditEvent) SetResourceType(v string)`

SetResourceType sets ResourceType field to given value.


### GetTenantId

`func (o *AuditEvent) GetTenantId() string`

GetTenantId returns the TenantId field if non-nil, zero value otherwise.

### GetTenantIdOk

`func (o *AuditEvent) GetTenantIdOk() (*string, bool)`

GetTenantIdOk returns a tuple with the TenantId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetTenantId

`func (o *AuditEvent) SetTenantId(v string)`

SetTenantId sets TenantId field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
