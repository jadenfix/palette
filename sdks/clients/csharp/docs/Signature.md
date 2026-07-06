# Beater.Client.Model.Signature
A structural fingerprint of a trace's failure shape.  Two traces with the same ordered failing-span shingles share a [`Signature`] (and therefore the same [`Signature::hash`]). The `shingles` set is also used for Jaccard similarity during clustering.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Hash** | **string** | Stable sha256 hash of the ordered shingles. |
**Shingles** | **List&lt;string&gt;** | Ordered &#x60;(kind|status)&#x60; shingles of failing spans. |

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
