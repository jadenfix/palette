# BeaterClient::SubmitReviewAnnotationHttpRequest

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **annotation_id** | **String** |  | [optional] |
| **payload** | **Object** |  |  |
| **reviewer_id** | **String** |  |  |
| **verdict** | [**ReviewVerdict**](ReviewVerdict.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::SubmitReviewAnnotationHttpRequest.new(
  annotation_id: null,
  payload: null,
  reviewer_id: null,
  verdict: null
)
```
