# ImportSourceHttpRequest

Request body for the unified import endpoint. The `source` field selects a registered [`beater_ingest::SourceImporter`] (e.g. `temporal_history`, `native`); `payload` is that source's document (Temporal `History` JSON, a native span list, …). Everything flows through the same downstream ingest pipeline as OTLP — there are no source-specific routes.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**payload** | **object** |  | [optional] 
**source** | **str** | Registered importer key, e.g. &#x60;temporal_history&#x60; or &#x60;native&#x60;. | 

## Example

```python
from beater_client.models.import_source_http_request import ImportSourceHttpRequest

# TODO update the JSON string below
json = "{}"
# create an instance of ImportSourceHttpRequest from a JSON string
import_source_http_request_instance = ImportSourceHttpRequest.from_json(json)
# print the JSON string representation of the object
print(ImportSourceHttpRequest.to_json())

# convert the object into a dict
import_source_http_request_dict = import_source_http_request_instance.to_dict()
# create an instance of ImportSourceHttpRequest from a dict
import_source_http_request_from_dict = ImportSourceHttpRequest.from_dict(import_source_http_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


