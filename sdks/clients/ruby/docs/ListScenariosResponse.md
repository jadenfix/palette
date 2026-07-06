# BeaterClient::ListScenariosResponse

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **next_cursor** | **String** |  | [optional] |
| **scenarios** | [**Array&lt;Scenario&gt;**](Scenario.md) |  |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::ListScenariosResponse.new(
  next_cursor: null,
  scenarios: null
)
```
