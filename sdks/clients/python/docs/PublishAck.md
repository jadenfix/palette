# PublishAck


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**accepted** | **bool** |  |
**duplicate** | **bool** |  |

## Example

```python
from beater_client.models.publish_ack import PublishAck

# TODO update the JSON string below
json = "{}"
# create an instance of PublishAck from a JSON string
publish_ack_instance = PublishAck.from_json(json)
# print the JSON string representation of the object
print(PublishAck.to_json())

# convert the object into a dict
publish_ack_dict = publish_ack_instance.to_dict()
# create an instance of PublishAck from a dict
publish_ack_from_dict = PublishAck.from_dict(publish_ack_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
