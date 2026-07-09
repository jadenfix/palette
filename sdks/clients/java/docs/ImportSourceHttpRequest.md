

# ImportSourceHttpRequest

Request body for the unified import endpoint. The `source` field selects a registered [`beater_ingest::SourceImporter`] (e.g. `temporal_history`, `native`); `payload` is that source's document (Temporal `History` JSON, a native span list, …). Everything flows through the same downstream ingest pipeline as OTLP — there are no source-specific routes.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**payload** | **Object** |  |  [optional] |
|**source** | **String** | Registered importer key, e.g. &#x60;temporal_history&#x60; or &#x60;native&#x60;. |  |
