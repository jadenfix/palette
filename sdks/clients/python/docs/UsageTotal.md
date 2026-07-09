# UsageTotal


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**quantity** | **int** |  |
**unit** | **str** |  |

## Example

```python
from beater_client.models.usage_total import UsageTotal

# TODO update the JSON string below
json = "{}"
# create an instance of UsageTotal from a JSON string
usage_total_instance = UsageTotal.from_json(json)
# print the JSON string representation of the object
print(UsageTotal.to_json())

# convert the object into a dict
usage_total_dict = usage_total_instance.to_dict()
# create an instance of UsageTotal from a dict
usage_total_from_dict = UsageTotal.from_dict(usage_total_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
