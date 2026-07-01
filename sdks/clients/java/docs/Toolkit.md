

# Toolkit

A connectable third-party app (Composio \"toolkit\"), flattened from the v3 `GET /toolkits` shape into the fields Beater exposes.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**authSchemes** | **List&lt;String&gt;** | Supported auth schemes (e.g. &#x60;OAUTH2&#x60;, &#x60;API_KEY&#x60;, &#x60;NO_AUTH&#x60;). |  [optional] |
|**description** | **String** | Short description, if the catalog provides one. |  [optional] |
|**name** | **String** | Human display name. |  |
|**noAuth** | **Boolean** | &#x60;true&#x60; when the toolkit needs no OAuth/connection to execute. |  [optional] |
|**slug** | **String** | Stable slug used everywhere else (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;). |  |
|**toolsCount** | **Integer** | Number of tools the toolkit exposes, if known. |  [optional] |



