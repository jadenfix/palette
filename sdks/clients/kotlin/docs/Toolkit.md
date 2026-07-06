
# Toolkit

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **name** | **kotlin.String** | Human display name. |  |
| **slug** | **kotlin.String** | Stable slug used everywhere else (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;). |  |
| **authSchemes** | **kotlin.collections.List&lt;kotlin.String&gt;** | Supported auth schemes (e.g. &#x60;OAUTH2&#x60;, &#x60;API_KEY&#x60;, &#x60;NO_AUTH&#x60;). |  [optional] |
| **description** | **kotlin.String** | Short description, if the catalog provides one. |  [optional] |
| **noAuth** | **kotlin.Boolean** | &#x60;true&#x60; when the toolkit needs no OAuth/connection to execute. |  [optional] |
| **toolsCount** | **kotlin.Int** | Number of tools the toolkit exposes, if known. |  [optional] |
