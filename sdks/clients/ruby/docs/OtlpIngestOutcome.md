# BeaterClient::OtlpIngestOutcome

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **accepted_raw** | **Integer** |  |  |
| **accepted_spans** | **Integer** |  |  |
| **downstream_queued** | **Boolean** |  |  |
| **duplicate_raw** | **Integer** |  |  |
| **duplicate_spans** | **Integer** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::OtlpIngestOutcome.new(
  accepted_raw: null,
  accepted_spans: null,
  downstream_queued: null,
  duplicate_raw: null,
  duplicate_spans: null
)
```
