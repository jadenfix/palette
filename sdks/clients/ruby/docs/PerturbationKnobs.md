# BeaterClient::PerturbationKnobs

## Properties

| Name | Type | Description | Notes |
| ---- | ---- | ----------- | ----- |
| **auth_failure** | **Boolean** | Force an auth failure on a dependency. |  |
| **contradictory_source** | **Boolean** | Inject a contradictory context source. |  |
| **prompt_injection** | **Boolean** | Attempt a prompt-injection payload. |  |
| **stale_source** | **Boolean** | Serve a stale version of a context source. |  |
| **timeout** | **Boolean** | Force a timeout on a dependency. |  |
| **tool_schema_mismatch** | **Boolean** | Present a tool whose schema mismatches expectations. |  |

## Example

```ruby
require 'beater_client'

instance = BeaterClient::PerturbationKnobs.new(
  auth_failure: null,
  contradictory_source: null,
  prompt_injection: null,
  stale_source: null,
  timeout: null,
  tool_schema_mismatch: null
)
```
