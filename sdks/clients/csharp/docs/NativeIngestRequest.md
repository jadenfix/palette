# Beater.Client.Model.NativeIngestRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Attributes** | **Dictionary&lt;string, Object&gt;** |  |
**AuthContext** | [**AuthContext**](AuthContext.md) |  | [optional]
**Cost** | [**Money**](Money.md) |  | [optional]
**EndTime** | **DateTime?** |  | [optional]
**IdempotencyKey** | **string** |  | [optional]
**Input** | **Object** |  | [optional]
**Kind** | **string** | Canonical agent span kind such as agent.run or llm.call |
**Model** | [**ModelRef**](ModelRef.md) |  | [optional]
**Name** | **string** |  |
**Output** | **Object** |  | [optional]
**ParentSpanId** | **string** |  | [optional]
**RedactionClass** | **RedactionClass** |  |
**Scope** | [**TenantScope**](TenantScope.md) |  |
**Seq** | **long** |  |
**SpanId** | **string** |  |
**StartTime** | **DateTime?** |  | [optional]
**Status** | **SpanStatus** |  |
**Tokens** | [**TokenCounts**](TokenCounts.md) |  | [optional]
**TraceId** | **string** |  |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
