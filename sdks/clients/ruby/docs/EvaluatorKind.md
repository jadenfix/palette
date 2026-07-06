# BeaterClient::EvaluatorKind

## Class instance methods

### `openapi_one_of`

Returns the list of classes defined in oneOf.

#### Example

```ruby
require 'beater_client'

BeaterClient::EvaluatorKind.openapi_one_of
# =>
# [
#   :'EvaluatorKindOneOf',
#   :'EvaluatorKindOneOf1',
#   :'EvaluatorKindOneOf10',
#   :'EvaluatorKindOneOf2',
#   :'EvaluatorKindOneOf3',
#   :'EvaluatorKindOneOf4',
#   :'EvaluatorKindOneOf5',
#   :'EvaluatorKindOneOf6',
#   :'EvaluatorKindOneOf7',
#   :'EvaluatorKindOneOf8',
#   :'EvaluatorKindOneOf9'
# ]
```

### build

Find the appropriate object from the `openapi_one_of` list and casts the data into it.

#### Example

```ruby
require 'beater_client'

BeaterClient::EvaluatorKind.build(data)
# => #<EvaluatorKindOneOf:0x00007fdd4aab02a0>

BeaterClient::EvaluatorKind.build(data_that_doesnt_match)
# => nil
```

#### Parameters

| Name | Type | Description |
| ---- | ---- | ----------- |
| **data** | **Mixed** | data to be matched against the list of oneOf items |

#### Return type

- `EvaluatorKindOneOf`
- `EvaluatorKindOneOf1`
- `EvaluatorKindOneOf10`
- `EvaluatorKindOneOf2`
- `EvaluatorKindOneOf3`
- `EvaluatorKindOneOf4`
- `EvaluatorKindOneOf5`
- `EvaluatorKindOneOf6`
- `EvaluatorKindOneOf7`
- `EvaluatorKindOneOf8`
- `EvaluatorKindOneOf9`
- `nil` (if no type matches)
