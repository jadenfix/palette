# DeadLetter


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**failed_at** | **datetime** |  |
**message** | [**BusMessage**](BusMessage.md) |  |
**reason** | **str** |  |

## Example

```python
from beater_client.models.dead_letter import DeadLetter

# TODO update the JSON string below
json = "{}"
# create an instance of DeadLetter from a JSON string
dead_letter_instance = DeadLetter.from_json(json)
# print the JSON string representation of the object
print(DeadLetter.to_json())

# convert the object into a dict
dead_letter_dict = dead_letter_instance.to_dict()
# create an instance of DeadLetter from a dict
dead_letter_from_dict = DeadLetter.from_dict(dead_letter_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
