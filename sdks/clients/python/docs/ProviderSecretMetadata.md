# ProviderSecretMetadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**active** | **bool** |  |
**created_at** | **datetime** |  |
**display_name** | **str** |  |
**project_id** | **str** |  |
**provider** | **str** |  |
**provider_secret_id** | **str** |  |
**rotated_at** | **datetime** |  | [optional]
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.provider_secret_metadata import ProviderSecretMetadata

# TODO update the JSON string below
json = "{}"
# create an instance of ProviderSecretMetadata from a JSON string
provider_secret_metadata_instance = ProviderSecretMetadata.from_json(json)
# print the JSON string representation of the object
print(ProviderSecretMetadata.to_json())

# convert the object into a dict
provider_secret_metadata_dict = provider_secret_metadata_instance.to_dict()
# create an instance of ProviderSecretMetadata from a dict
provider_secret_metadata_from_dict = ProviderSecretMetadata.from_dict(provider_secret_metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
