# BeaterClient::Toolkit

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **auth_schemes** | **Array&lt;String&gt;** | Supported auth schemes (e.g. &#x60;OAUTH2&#x60;, &#x60;API_KEY&#x60;, &#x60;NO_AUTH&#x60;). | [optional] |
| **description** | **String** | Short description, if the catalog provides one. | [optional] |
| **name** | **String** | Human display name. |  |
| **no_auth** | **Boolean** | &#x60;true&#x60; when the toolkit needs no OAuth/connection to execute. | [optional] |
| **slug** | **String** | Stable slug used everywhere else (e.g. &#x60;github&#x60;, &#x60;gmail&#x60;). |  |
| **tools_count** | **Integer** | Number of tools the toolkit exposes, if known. | [optional] |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::Toolkit.new(
  auth_schemes: null,
  description: null,
  name: null,
  no_auth: null,
  slug: null,
  tools_count: null
)
```
