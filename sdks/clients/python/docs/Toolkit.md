# Toolkit

A connectable third-party app (Composio \"toolkit\"), flattened from the v3 `GET /toolkits` shape into the fields Beater exposes.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**auth_schemes** | **List[str]** | Supported auth schemes (e.g. &#x60;OAUTH2&#x60;, &#x60;API_KEY&#x60;, &#x60;NO_AUTH&#x60;). | [optional]
**description** | **str** | Short description, if the catalog provides one. | [optional]
**name** | **str** | Human display name. |
**no_auth** | **bool** | &#x60;true&#x60; when the toolkit needs no OAuth/connection to execute. | [optional]
**slug** | **str** | Stable slug used everywhere else (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;). |
**tools_count** | **int** | Number of tools the toolkit exposes, if known. | [optional]

## Example

```python
from beater_client.models.toolkit import Toolkit

# TODO update the JSON string below
json = "{}"
# create an instance of Toolkit from a JSON string
toolkit_instance = Toolkit.from_json(json)
# print the JSON string representation of the object
print(Toolkit.to_json())

# convert the object into a dict
toolkit_dict = toolkit_instance.to_dict()
# create an instance of Toolkit from a dict
toolkit_from_dict = Toolkit.from_dict(toolkit_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
