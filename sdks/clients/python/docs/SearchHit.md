# SearchHit


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**environment_id** | **str** |  |
**kind** | **str** |  |
**model** | **str** |  |
**name** | **str** |  |
**project_id** | **str** |  |
**score** | **float** |  |
**span_id** | **str** |  |
**status** | **str** |  |
**tenant_id** | **str** |  |
**tool** | **str** |  |
**trace_id** | **str** |  |

## Example

```python
from beater_client.models.search_hit import SearchHit

# TODO update the JSON string below
json = "{}"
# create an instance of SearchHit from a JSON string
search_hit_instance = SearchHit.from_json(json)
# print the JSON string representation of the object
print(SearchHit.to_json())

# convert the object into a dict
search_hit_dict = search_hit_instance.to_dict()
# create an instance of SearchHit from a dict
search_hit_from_dict = SearchHit.from_dict(search_hit_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
