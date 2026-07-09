# ScoreResult


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**evidence** | **object** |  |
**label** | **str** |  | [optional]
**score** | **float** |  |

## Example

```python
from beater_client.models.score_result import ScoreResult

# TODO update the JSON string below
json = "{}"
# create an instance of ScoreResult from a JSON string
score_result_instance = ScoreResult.from_json(json)
# print the JSON string representation of the object
print(ScoreResult.to_json())

# convert the object into a dict
score_result_dict = score_result_instance.to_dict()
# create an instance of ScoreResult from a dict
score_result_from_dict = ScoreResult.from_dict(score_result_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
