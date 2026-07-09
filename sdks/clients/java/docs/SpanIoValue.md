

# SpanIoValue

## oneOf schemas
* [SpanIoValueOneOf](SpanIoValueOneOf.md)
* [SpanIoValueOneOf1](SpanIoValueOneOf1.md)
* [SpanIoValueOneOf2](SpanIoValueOneOf2.md)
* [SpanIoValueOneOf3](SpanIoValueOneOf3.md)

## Example
```java
// Import classes:
import ai.beater.client.model.SpanIoValue;
import ai.beater.client.model.SpanIoValueOneOf;
import ai.beater.client.model.SpanIoValueOneOf1;
import ai.beater.client.model.SpanIoValueOneOf2;
import ai.beater.client.model.SpanIoValueOneOf3;

public class Example {
    public static void main(String[] args) {
        SpanIoValue exampleSpanIoValue = new SpanIoValue();

        // create a new SpanIoValueOneOf
        SpanIoValueOneOf exampleSpanIoValueOneOf = new SpanIoValueOneOf();
        // set SpanIoValue to SpanIoValueOneOf
        exampleSpanIoValue.setActualInstance(exampleSpanIoValueOneOf);
        // to get back the SpanIoValueOneOf set earlier
        SpanIoValueOneOf testSpanIoValueOneOf = (SpanIoValueOneOf) exampleSpanIoValue.getActualInstance();

        // create a new SpanIoValueOneOf1
        SpanIoValueOneOf1 exampleSpanIoValueOneOf1 = new SpanIoValueOneOf1();
        // set SpanIoValue to SpanIoValueOneOf1
        exampleSpanIoValue.setActualInstance(exampleSpanIoValueOneOf1);
        // to get back the SpanIoValueOneOf1 set earlier
        SpanIoValueOneOf1 testSpanIoValueOneOf1 = (SpanIoValueOneOf1) exampleSpanIoValue.getActualInstance();

        // create a new SpanIoValueOneOf2
        SpanIoValueOneOf2 exampleSpanIoValueOneOf2 = new SpanIoValueOneOf2();
        // set SpanIoValue to SpanIoValueOneOf2
        exampleSpanIoValue.setActualInstance(exampleSpanIoValueOneOf2);
        // to get back the SpanIoValueOneOf2 set earlier
        SpanIoValueOneOf2 testSpanIoValueOneOf2 = (SpanIoValueOneOf2) exampleSpanIoValue.getActualInstance();

        // create a new SpanIoValueOneOf3
        SpanIoValueOneOf3 exampleSpanIoValueOneOf3 = new SpanIoValueOneOf3();
        // set SpanIoValue to SpanIoValueOneOf3
        exampleSpanIoValue.setActualInstance(exampleSpanIoValueOneOf3);
        // to get back the SpanIoValueOneOf3 set earlier
        SpanIoValueOneOf3 testSpanIoValueOneOf3 = (SpanIoValueOneOf3) exampleSpanIoValue.getActualInstance();
    }
}
```
