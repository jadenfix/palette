# BusMessage


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**attempts** | **int** |  |
**enqueued_at** | **datetime** |  |
**idempotency_key** | **str** |  |
**kind** | **str** |  |
**max_attempts** | **int** |  |
**message_id** | **str** |  |
**payload** | **List[int]** |  |
**project_id** | **str** |  |
**tenant_id** | **str** |  |

## Example

```python
from beater_client.models.bus_message import BusMessage

# TODO update the JSON string below
json = "{}"
# create an instance of BusMessage from a JSON string
bus_message_instance = BusMessage.from_json(json)
# print the JSON string representation of the object
print(BusMessage.to_json())

# convert the object into a dict
bus_message_dict = bus_message_instance.to_dict()
# create an instance of BusMessage from a dict
bus_message_from_dict = BusMessage.from_dict(bus_message_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
