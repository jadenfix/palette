# ConnectorSkillsResponse

Generated prompting scaffold (\"skills.md\") for a toolkit's tools.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**skills** | **str** | Markdown document: one skill card per tool, ready to splice into an agent&#39;s system prompt. | 
**toolkit** | **str** | Toolkit the skills document covers. | 

## Example

```python
from beater_client.models.connector_skills_response import ConnectorSkillsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ConnectorSkillsResponse from a JSON string
connector_skills_response_instance = ConnectorSkillsResponse.from_json(json)
# print the JSON string representation of the object
print(ConnectorSkillsResponse.to_json())

# convert the object into a dict
connector_skills_response_dict = connector_skills_response_instance.to_dict()
# create an instance of ConnectorSkillsResponse from a dict
connector_skills_response_from_dict = ConnectorSkillsResponse.from_dict(connector_skills_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


