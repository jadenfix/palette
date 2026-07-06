# BeaterClient::ArtifactRef

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **artifact_id** | **String** |  |  |
| **mime_type** | **String** |  |  |
| **redaction_class** | [**RedactionClass**](RedactionClass.md) |  |  |
| **sha256** | **String** |  |  |
| **size_bytes** | **Integer** |  |  |
| **uri** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ArtifactRef.new(
  artifact_id: null,
  mime_type: null,
  redaction_class: null,
  sha256: null,
  size_bytes: null,
  uri: null
)
```
