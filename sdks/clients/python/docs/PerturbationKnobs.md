# PerturbationKnobs

Tunable knobs describing how a scenario may be perturbed during replay.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**auth_failure** | **bool** | Force an auth failure on a dependency. |
**contradictory_source** | **bool** | Inject a contradictory context source. |
**prompt_injection** | **bool** | Attempt a prompt-injection payload. |
**stale_source** | **bool** | Serve a stale version of a context source. |
**timeout** | **bool** | Force a timeout on a dependency. |
**tool_schema_mismatch** | **bool** | Present a tool whose schema mismatches expectations. |

## Example

```python
from beater_client.models.perturbation_knobs import PerturbationKnobs

# TODO update the JSON string below
json = "{}"
# create an instance of PerturbationKnobs from a JSON string
perturbation_knobs_instance = PerturbationKnobs.from_json(json)
# print the JSON string representation of the object
print(PerturbationKnobs.to_json())

# convert the object into a dict
perturbation_knobs_dict = perturbation_knobs_instance.to_dict()
# create an instance of PerturbationKnobs from a dict
perturbation_knobs_from_dict = PerturbationKnobs.from_dict(perturbation_knobs_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
