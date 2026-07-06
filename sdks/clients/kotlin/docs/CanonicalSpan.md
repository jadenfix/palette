
# CanonicalSpan

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **attributes** | [**kotlin.collections.Map&lt;kotlin.String, kotlin.Any&gt;**](kotlin.Any.md) |  |  |
| **environmentId** | **kotlin.String** |  |  |
| **kind** | **kotlin.String** | Canonical agent span kind such as agent.run or llm.call |  |
| **name** | **kotlin.String** |  |  |
| **normalizerVersion** | **kotlin.String** |  |  |
| **projectId** | **kotlin.String** |  |  |
| **rawRef** | [**ArtifactRef**](ArtifactRef.md) |  |  |
| **schemaVersion** | **kotlin.Int** |  |  |
| **seq** | **kotlin.Long** |  |  |
| **spanId** | **kotlin.String** |  |  |
| **startTime** | [**java.time.OffsetDateTime**](java.time.OffsetDateTime.md) |  |  |
| **status** | [**SpanStatus**](SpanStatus.md) |  |  |
| **tenantId** | **kotlin.String** |  |  |
| **traceId** | **kotlin.String** |  |  |
| **unmappedAttrs** | [**kotlin.Any**](.md) |  |  |
| **cost** | [**Money**](Money.md) |  |  [optional] |
| **endTime** | [**java.time.OffsetDateTime**](java.time.OffsetDateTime.md) |  |  [optional] |
| **inputRef** | [**ArtifactRef**](ArtifactRef.md) |  |  [optional] |
| **model** | [**ModelRef**](ModelRef.md) |  |  [optional] |
| **outputRef** | [**ArtifactRef**](ArtifactRef.md) |  |  [optional] |
| **parentSpanId** | **kotlin.String** |  |  [optional] |
| **tokens** | [**TokenCounts**](TokenCounts.md) |  |  [optional] |
