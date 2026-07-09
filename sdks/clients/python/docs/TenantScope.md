# TenantScope


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**environment_id** | **str** |  |
**project_id** | **str** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.tenant_scope import TenantScope

# TODO update the JSON string below
json = "{}"
# create an instance of TenantScope from a JSON string
tenant_scope_instance = TenantScope.from_json(json)
# print the JSON string representation of the object
print(TenantScope.to_json())

# convert the object into a dict
tenant_scope_dict = tenant_scope_instance.to_dict()
# create an instance of TenantScope from a dict
tenant_scope_from_dict = TenantScope.from_dict(tenant_scope_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
