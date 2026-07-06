# BeaterClient::SpanIoValue

## Class instance methods

### `openapi_one_of`

Returns the list of classes defined in oneOf.

#### Example

```ruby
require 'beater_client'

BeaterClient::SpanIoValue.openapi_one_of
# =>
# [
#   :'SpanIoValueOneOf',
#   :'SpanIoValueOneOf1',
#   :'SpanIoValueOneOf2',
#   :'SpanIoValueOneOf3'
# ]
```

### build

Find the appropriate object from the `openapi_one_of` list and casts the data into it.

#### Example

```ruby
require 'beater_client'

BeaterClient::SpanIoValue.build(data)
# => #<SpanIoValueOneOf:0x00007fdd4aab02a0>

BeaterClient::SpanIoValue.build(data_that_doesnt_match)
# => nil
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| **data** | **Mixed** | data to be matched against the list of oneOf items |

#### Return type

- `SpanIoValueOneOf`
- `SpanIoValueOneOf1`
- `SpanIoValueOneOf2`
- `SpanIoValueOneOf3`
- `nil` (if no type matches)
