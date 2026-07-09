

# CanonicalSpan


## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**attributes** | **Map&lt;String, Object&gt;** |  |  |
|**cost** | [**Money**](Money.md) |  |  [optional] |
|**endTime** | **OffsetDateTime** |  |  [optional] |
|**environmentId** | **String** |  |  |
|**inputRef** | [**ArtifactRef**](ArtifactRef.md) |  |  [optional] |
|**kind** | **String** | Canonical agent span kind such as agent.run or llm.call |  |
|**model** | [**ModelRef**](ModelRef.md) |  |  [optional] |
|**name** | **String** |  |  |
|**normalizerVersion** | **String** |  |  |
|**outputRef** | [**ArtifactRef**](ArtifactRef.md) |  |  [optional] |
|**parentSpanId** | **String** |  |  [optional] |
|**projectId** | **String** |  |  |
|**rawRef** | [**ArtifactRef**](ArtifactRef.md) |  |  |
|**schemaVersion** | **Integer** |  |  |
|**seq** | **Long** |  |  |
|**spanId** | **String** |  |  |
|**startTime** | **OffsetDateTime** |  |  |
|**status** | **SpanStatus** |  |  |
|**tenantId** | **String** |  |  |
|**tokens** | [**TokenCounts**](TokenCounts.md) |  |  [optional] |
|**traceId** | **String** |  |  |
|**unmappedAttrs** | **Object** |  |  |
