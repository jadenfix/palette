

# ConnectionStatus

Connection status of one app for one entity.

## Properties

| Name | Type | Description | Notes |
|------------ | ------------- | ------------- | -------------|
|**connected** | **Boolean** | &#x60;true&#x60; only when an account exists and is &#x60;ACTIVE&#x60;. |  |
|**connectedAccountId** | **String** | The connected-account id, when one exists. |  [optional] |
|**status** | **String** | Raw Composio status (&#x60;ACTIVE&#x60;, &#x60;INITIALIZING&#x60;, &#x60;FAILED&#x60;, …) or &#x60;not_connected&#x60; when no account exists yet. |  |
|**toolkit** | **String** | Toolkit slug this status is for. |  |
