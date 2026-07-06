# BeaterClient::DatasetVersionSnapshot

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **cases** | [**Array&lt;DatasetCase&gt;**](DatasetCase.md) |  |  |
| **corpus_root** | **String** | A content-addressed Merkle root naming the exact contents of a corpus.  Serialized as its lowercase-hex SHA-256 string. |  |
| **created_at** | **Time** |  |  |
| **dataset_id** | **String** |  |  |
| **project_id** | **String** |  |  |
| **tenant_id** | **String** |  |  |
| **version_id** | **String** |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::DatasetVersionSnapshot.new(
  cases: null,
  corpus_root: null,
  created_at: null,
  dataset_id: null,
  project_id: null,
  tenant_id: null,
  version_id: null
)
```
