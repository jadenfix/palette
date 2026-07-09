

# NativeIngestRequest


## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**attributes** | **Map&lt;String, Object&gt;** |  |  |
|**authContext** | [**AuthContext**](AuthContext.md) |  |  [optional] |
|**cost** | [**Money**](Money.md) |  |  [optional] |
|**endTime** | **OffsetDateTime** |  |  [optional] |
|**idempotencyKey** | **String** |  |  [optional] |
|**input** | **Object** |  |  [optional] |
|**kind** | **String** | Canonical agent span kind such as agent.run or llm.call |  |
|**model** | [**ModelRef**](ModelRef.md) |  |  [optional] |
|**name** | **String** |  |  |
|**output** | **Object** |  |  [optional] |
|**parentSpanId** | **String** |  |  [optional] |
|**redactionClass** | **RedactionClass** |  |  |
|**scope** | [**TenantScope**](TenantScope.md) |  |  |
|**seq** | **Long** |  |  |
|**spanId** | **String** |  |  |
|**startTime** | **OffsetDateTime** |  |  [optional] |
|**status** | **SpanStatus** |  |  |
|**tokens** | [**TokenCounts**](TokenCounts.md) |  |  [optional] |
|**traceId** | **String** |  |  |
