# RevokedApiKey


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**active** | **bool** |  |
**api_key_id** | **str** |  |
**rotated_at** | **datetime** |  |

## Example

```python
from beater_client.models.revoked_api_key import RevokedApiKey

# TODO update the JSON string below
json = "{}"
# create an instance of RevokedApiKey from a JSON string
revoked_api_key_instance = RevokedApiKey.from_json(json)
# print the JSON string representation of the object
print(RevokedApiKey.to_json())

# convert the object into a dict
revoked_api_key_dict = revoked_api_key_instance.to_dict()
# create an instance of RevokedApiKey from a dict
revoked_api_key_from_dict = RevokedApiKey.from_dict(revoked_api_key_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
