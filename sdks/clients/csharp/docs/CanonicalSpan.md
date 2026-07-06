# Beater.Client.Model.CanonicalSpan

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Attributes** | **Dictionary&lt;string, Object&gt;** |  |
**Cost** | [**Money**](Money.md) |  | [optional]
**EndTime** | **DateTime?** |  | [optional]
**EnvironmentId** | **string** |  |
**InputRef** | [**ArtifactRef**](ArtifactRef.md) |  | [optional]
**Kind** | **string** | Canonical agent span kind such as agent.run or llm.call |
**Model** | [**ModelRef**](ModelRef.md) |  | [optional]
**Name** | **string** |  |
**NormalizerVersion** | **string** |  |
**OutputRef** | [**ArtifactRef**](ArtifactRef.md) |  | [optional]
**ParentSpanId** | **string** |  | [optional]
**ProjectId** | **string** |  |
**RawRef** | [**ArtifactRef**](ArtifactRef.md) |  |
**SchemaVersion** | **int** |  |
**Seq** | **long** |  |
**SpanId** | **string** |  |
**StartTime** | **DateTime** |  |
**Status** | **SpanStatus** |  |
**TenantId** | **string** |  |
**Tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional]
**TraceId** | **string** |  |
**UnmappedAttrs** | **Object** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
