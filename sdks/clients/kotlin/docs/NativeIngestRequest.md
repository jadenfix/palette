
# NativeIngestRequest

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **attributes** | [**kotlin.collections.Map&lt;kotlin.String, kotlin.Any&gt;**](kotlin.Any.md) |  |  |
| **kind** | **kotlin.String** | Canonical agent span kind such as agent.run or llm.call |  |
| **name** | **kotlin.String** |  |  |
| **redactionClass** | [**RedactionClass**](RedactionClass.md) |  |  |
| **scope** | [**TenantScope**](TenantScope.md) |  |  |
| **seq** | **kotlin.Long** |  |  |
| **spanId** | **kotlin.String** |  |  |
| **status** | [**SpanStatus**](SpanStatus.md) |  |  |
| **traceId** | **kotlin.String** |  |  |
| **authContext** | [**AuthContext**](AuthContext.md) |  |  [optional] |
| **cost** | [**Money**](Money.md) |  |  [optional] |
| **endTime** | [**java.time.OffsetDateTime**](java.time.OffsetDateTime.md) |  |  [optional] |
| **idempotencyKey** | **kotlin.String** |  |  [optional] |
| **input** | [**kotlin.Any**](.md) |  |  [optional] |
| **model** | [**ModelRef**](ModelRef.md) |  |  [optional] |
| **output** | [**kotlin.Any**](.md) |  |  [optional] |
| **parentSpanId** | **kotlin.String** |  |  [optional] |
| **startTime** | [**java.time.OffsetDateTime**](java.time.OffsetDateTime.md) |  |  [optional] |
| **tokens** | [**TokenCounts**](TokenCounts.md) |  |  [optional] |
