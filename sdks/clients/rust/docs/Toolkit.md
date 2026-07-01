# Toolkit

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**auth_schemes** | Option<**Vec<String>**> | Supported auth schemes (e.g. `OAUTH2`, `API_KEY`, `NO_AUTH`). | [optional]
**description** | Option<**String**> | Short description, if the catalog provides one. | [optional]
**name** | **String** | Human display name. | 
**no_auth** | Option<**bool**> | `true` when the toolkit needs no OAuth/connection to execute. | [optional]
**slug** | **String** | Stable slug used everywhere else (e.g. `github`, `gmail`). | 
**tools_count** | Option<**i32**> | Number of tools the toolkit exposes, if known. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


