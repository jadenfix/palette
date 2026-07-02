

# Signature

A structural fingerprint of a trace's failure shape.  Two traces with the same ordered failing-span shingles share a [`Signature`] (and therefore the same [`Signature::hash`]). The `shingles` set is also used for Jaccard similarity during clustering.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**hash** | **String** | Stable sha256 hash of the ordered shingles. |  |
|**shingles** | **List&lt;String&gt;** | Ordered &#x60;(kind|status)&#x60; shingles of failing spans. |  |



