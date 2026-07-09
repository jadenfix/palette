# ArchiveQueryResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**rows** | [**List[ArchivedSpanRow]**](ArchivedSpanRow.md) |  |

## Example

```python
from beater_client.models.archive_query_response import ArchiveQueryResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ArchiveQueryResponse from a JSON string
archive_query_response_instance = ArchiveQueryResponse.from_json(json)
# print the JSON string representation of the object
print(ArchiveQueryResponse.to_json())

# convert the object into a dict
archive_query_response_dict = archive_query_response_instance.to_dict()
# create an instance of ArchiveQueryResponse from a dict
archive_query_response_from_dict = ArchiveQueryResponse.from_dict(archive_query_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
