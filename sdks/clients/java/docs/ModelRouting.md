

# ModelRouting

Per-request credential routing policy.  This enum is the encoding of *\"any model, optionally BYOK, else managed default\"*:  * [`ModelRouting::Byok`] — use the tenant's own opaque provider secret. The   model named in the [`ChatCompletionRequest`] is used as-is, so any model the   key can reach is reachable. * [`ModelRouting::Managed`] — use Beater's managed credentials with the given   default model. Only available when the gateway is built with a   [`ManagedDefault`] (hosted edition).  A [`GatewayRequest`] carries an *ordered* `Vec<ModelRouting>` so the gateway can fail over from one key/provider to the next. An empty list means *\"I did not bring a key\"*: the gateway then uses the managed default if configured, or returns [`GatewayError::NoKeyAndNoManagedDefault`] in OSS.

## oneOf schemas
* [ModelRoutingOneOf](ModelRoutingOneOf.md)
* [ModelRoutingOneOf1](ModelRoutingOneOf1.md)

## Example
```java
// Import classes:
import ai.beater.client.model.ModelRouting;
import ai.beater.client.model.ModelRoutingOneOf;
import ai.beater.client.model.ModelRoutingOneOf1;

public class Example {
    public static void main(String[] args) {
        ModelRouting exampleModelRouting = new ModelRouting();

        // create a new ModelRoutingOneOf
        ModelRoutingOneOf exampleModelRoutingOneOf = new ModelRoutingOneOf();
        // set ModelRouting to ModelRoutingOneOf
        exampleModelRouting.setActualInstance(exampleModelRoutingOneOf);
        // to get back the ModelRoutingOneOf set earlier
        ModelRoutingOneOf testModelRoutingOneOf = (ModelRoutingOneOf) exampleModelRouting.getActualInstance();

        // create a new ModelRoutingOneOf1
        ModelRoutingOneOf1 exampleModelRoutingOneOf1 = new ModelRoutingOneOf1();
        // set ModelRouting to ModelRoutingOneOf1
        exampleModelRouting.setActualInstance(exampleModelRoutingOneOf1);
        // to get back the ModelRoutingOneOf1 set earlier
        ModelRoutingOneOf1 testModelRoutingOneOf1 = (ModelRoutingOneOf1) exampleModelRouting.getActualInstance();
    }
}
```


