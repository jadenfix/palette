# ArtifactRef


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**artifact_id** | **str** |  |
**mime_type** | **str** |  |
**redaction_class** | [**RedactionClass**](RedactionClass.md) |  |
**sha256** | **str** |  |
**size_bytes** | **int** |  |
**uri** | **str** |  |

## Example

```python
from beater_client.models.artifact_ref import ArtifactRef

# TODO update the JSON string below
json = "{}"
# create an instance of ArtifactRef from a JSON string
artifact_ref_instance = ArtifactRef.from_json(json)
# print the JSON string representation of the object
print(ArtifactRef.to_json())

# convert the object into a dict
artifact_ref_dict = artifact_ref_instance.to_dict()
# create an instance of ArtifactRef from a dict
artifact_ref_from_dict = ArtifactRef.from_dict(artifact_ref_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
