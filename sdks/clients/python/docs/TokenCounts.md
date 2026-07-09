# TokenCounts


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cache_read** | **int** |  |
**input** | **int** |  |
**output** | **int** |  |
**reasoning** | **int** |  |

## Example

```python
from beater_client.models.token_counts import TokenCounts

# TODO update the JSON string below
json = "{}"
# create an instance of TokenCounts from a JSON string
token_counts_instance = TokenCounts.from_json(json)
# print the JSON string representation of the object
print(TokenCounts.to_json())

# convert the object into a dict
token_counts_dict = token_counts_instance.to_dict()
# create an instance of TokenCounts from a dict
token_counts_from_dict = TokenCounts.from_dict(token_counts_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
