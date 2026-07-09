# WriteAck


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**accepted_raw** | **int** |  |
**accepted_spans** | **int** |  |
**duplicate_raw** | **int** |  |
**duplicate_spans** | **int** |  |

## Example

```python
from beater_client.models.write_ack import WriteAck

# TODO update the JSON string below
json = "{}"
# create an instance of WriteAck from a JSON string
write_ack_instance = WriteAck.from_json(json)
# print the JSON string representation of the object
print(WriteAck.to_json())

# convert the object into a dict
write_ack_dict = write_ack_instance.to_dict()
# create an instance of WriteAck from a dict
write_ack_from_dict = WriteAck.from_dict(write_ack_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
