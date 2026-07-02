# Signature

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**Hash** | **string** | Stable sha256 hash of the ordered shingles. | 
**Shingles** | **[]string** | Ordered &#x60;(kind|status)&#x60; shingles of failing spans. | 

## Methods

### NewSignature

`func NewSignature(hash string, shingles []string, ) *Signature`

NewSignature instantiates a new Signature object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewSignatureWithDefaults

`func NewSignatureWithDefaults() *Signature`

NewSignatureWithDefaults instantiates a new Signature object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetHash

`func (o *Signature) GetHash() string`

GetHash returns the Hash field if non-nil, zero value otherwise.

### GetHashOk

`func (o *Signature) GetHashOk() (*string, bool)`

GetHashOk returns a tuple with the Hash field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetHash

`func (o *Signature) SetHash(v string)`

SetHash sets Hash field to given value.


### GetShingles

`func (o *Signature) GetShingles() []string`

GetShingles returns the Shingles field if non-nil, zero value otherwise.

### GetShinglesOk

`func (o *Signature) GetShinglesOk() (*[]string, bool)`

GetShinglesOk returns a tuple with the Shingles field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetShingles

`func (o *Signature) SetShingles(v []string)`

SetShingles sets Shingles field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


