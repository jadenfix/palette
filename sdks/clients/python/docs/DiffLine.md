# DiffLine


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**kind** | [**DiffLineKind**](DiffLineKind.md) |  |
**new_line** | **int** |  | [optional]
**old_line** | **int** |  | [optional]
**text** | **str** |  |

## Example

```python
from beater_client.models.diff_line import DiffLine

# TODO update the JSON string below
json = "{}"
# create an instance of DiffLine from a JSON string
diff_line_instance = DiffLine.from_json(json)
# print the JSON string representation of the object
print(DiffLine.to_json())

# convert the object into a dict
diff_line_dict = diff_line_instance.to_dict()
# create an instance of DiffLine from a dict
diff_line_from_dict = DiffLine.from_dict(diff_line_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
