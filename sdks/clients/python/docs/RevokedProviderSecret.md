# RevokedProviderSecret


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**active** | **bool** |  |
**provider_secret_id** | **str** |  |
**rotated_at** | **datetime** |  |

## Example

```python
from beater_client.models.revoked_provider_secret import RevokedProviderSecret

# TODO update the JSON string below
json = "{}"
# create an instance of RevokedProviderSecret from a JSON string
revoked_provider_secret_instance = RevokedProviderSecret.from_json(json)
# print the JSON string representation of the object
print(RevokedProviderSecret.to_json())

# convert the object into a dict
revoked_provider_secret_dict = revoked_provider_secret_instance.to_dict()
# create an instance of RevokedProviderSecret from a dict
revoked_provider_secret_from_dict = RevokedProviderSecret.from_dict(revoked_provider_secret_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
