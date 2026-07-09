# ArtifactRef

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ArtifactId** | **string** |  |
**MimeType** | **string** |  |
**RedactionClass** | [**RedactionClass**](RedactionClass.md) |  |
**Sha256** | **string** |  |
**SizeBytes** | **int64** |  |
**Uri** | **string** |  |

## Methods

### NewArtifactRef

`func NewArtifactRef(artifactId string, mimeType string, redactionClass RedactionClass, sha256 string, sizeBytes int64, uri string, ) *ArtifactRef`

NewArtifactRef instantiates a new ArtifactRef object
This constructor will assign default values to properties that have it defined,
and makes sure properties required by API are set, but the set of arguments
will change when the set of required properties is changed

### NewArtifactRefWithDefaults

`func NewArtifactRefWithDefaults() *ArtifactRef`

NewArtifactRefWithDefaults instantiates a new ArtifactRef object
This constructor will only assign default values to properties that have it defined,
but it doesn't guarantee that properties required by API are set

### GetArtifactId

`func (o *ArtifactRef) GetArtifactId() string`

GetArtifactId returns the ArtifactId field if non-nil, zero value otherwise.

### GetArtifactIdOk

`func (o *ArtifactRef) GetArtifactIdOk() (*string, bool)`

GetArtifactIdOk returns a tuple with the ArtifactId field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetArtifactId

`func (o *ArtifactRef) SetArtifactId(v string)`

SetArtifactId sets ArtifactId field to given value.


### GetMimeType

`func (o *ArtifactRef) GetMimeType() string`

GetMimeType returns the MimeType field if non-nil, zero value otherwise.

### GetMimeTypeOk

`func (o *ArtifactRef) GetMimeTypeOk() (*string, bool)`

GetMimeTypeOk returns a tuple with the MimeType field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetMimeType

`func (o *ArtifactRef) SetMimeType(v string)`

SetMimeType sets MimeType field to given value.


### GetRedactionClass

`func (o *ArtifactRef) GetRedactionClass() RedactionClass`

GetRedactionClass returns the RedactionClass field if non-nil, zero value otherwise.

### GetRedactionClassOk

`func (o *ArtifactRef) GetRedactionClassOk() (*RedactionClass, bool)`

GetRedactionClassOk returns a tuple with the RedactionClass field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetRedactionClass

`func (o *ArtifactRef) SetRedactionClass(v RedactionClass)`

SetRedactionClass sets RedactionClass field to given value.


### GetSha256

`func (o *ArtifactRef) GetSha256() string`

GetSha256 returns the Sha256 field if non-nil, zero value otherwise.

### GetSha256Ok

`func (o *ArtifactRef) GetSha256Ok() (*string, bool)`

GetSha256Ok returns a tuple with the Sha256 field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSha256

`func (o *ArtifactRef) SetSha256(v string)`

SetSha256 sets Sha256 field to given value.


### GetSizeBytes

`func (o *ArtifactRef) GetSizeBytes() int64`

GetSizeBytes returns the SizeBytes field if non-nil, zero value otherwise.

### GetSizeBytesOk

`func (o *ArtifactRef) GetSizeBytesOk() (*int64, bool)`

GetSizeBytesOk returns a tuple with the SizeBytes field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetSizeBytes

`func (o *ArtifactRef) SetSizeBytes(v int64)`

SetSizeBytes sets SizeBytes field to given value.


### GetUri

`func (o *ArtifactRef) GetUri() string`

GetUri returns the Uri field if non-nil, zero value otherwise.

### GetUriOk

`func (o *ArtifactRef) GetUriOk() (*string, bool)`

GetUriOk returns a tuple with the Uri field if it's non-nil, zero value otherwise
and a boolean to check if the value has been set.

### SetUri

`func (o *ArtifactRef) SetUri(v string)`

SetUri sets Uri field to given value.



[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
