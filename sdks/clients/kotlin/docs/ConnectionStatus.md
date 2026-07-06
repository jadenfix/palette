
# ConnectionStatus

## Properties
| Name | Type | Description | Notes |
| ------------ | ------------- | ------------- | ------------- |
| **connected** | **kotlin.Boolean** | &#x60;true&#x60; only when an account exists and is &#x60;ACTIVE&#x60;. |  |
| **status** | **kotlin.String** | Raw Composio status (&#x60;ACTIVE&#x60;, &#x60;INITIALIZING&#x60;, &#x60;FAILED&#x60;, …) or &#x60;not_connected&#x60; when no account exists yet. |  |
| **toolkit** | **kotlin.String** | Toolkit slug this status is for. |  |
| **connectedAccountId** | **kotlin.String** | The connected-account id, when one exists. |  [optional] |
