

# ConnectionLink

One-time login link returned when initiating a managed-OAuth connection.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**connectedAccountId** | **String** | Composio connection id (&#x60;ca_…&#x60;) created for this handshake. |  |
|**expiresAt** | **String** | When the link expires (RFC 3339), if provided. |  [optional] |
|**redirectUrl** | **String** | URL the end user opens once to authorize the app. |  |
