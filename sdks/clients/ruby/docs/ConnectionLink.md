# BeaterClient::ConnectionLink

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **connected_account_id** | **String** | Composio connection id (&#x60;ca_…&#x60;) created for this handshake. |  |
| **expires_at** | **String** | When the link expires (RFC 3339), if provided. | [optional] |
| **redirect_url** | **String** | URL the end user opens once to authorize the app. |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ConnectionLink.new(
  connected_account_id: null,
  expires_at: null,
  redirect_url: null
)
```
