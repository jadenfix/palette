# BeaterClient::Signature

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **hash** | **String** | Stable sha256 hash of the ordered shingles. |  |
| **shingles** | **Array&lt;String&gt;** | Ordered &#x60;(kind|status)&#x60; shingles of failing spans. |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::Signature.new(
  hash: null,
  shingles: null
)
```
