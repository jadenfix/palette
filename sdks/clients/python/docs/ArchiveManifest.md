# ArchiveManifest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  |
**path** | **str** |  |
**project_id** | **str** |  |
**span_count** | **int** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.archive_manifest import ArchiveManifest

# TODO update the JSON string below
json = "{}"
# create an instance of ArchiveManifest from a JSON string
archive_manifest_instance = ArchiveManifest.from_json(json)
# print the JSON string representation of the object
print(ArchiveManifest.to_json())

# convert the object into a dict
archive_manifest_dict = archive_manifest_instance.to_dict()
# create an instance of ArchiveManifest from a dict
archive_manifest_from_dict = ArchiveManifest.from_dict(archive_manifest_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
