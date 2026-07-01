#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>
#include "ConnectorsAPI.h"

#define MAX_NUMBER_LENGTH 16
#define MAX_BUFFER_LENGTH 4096


connection_link_t*
ConnectorsAPI_connectConnector(apiClient_t *apiClient, char *tenant_id, char *project_id, connect_connector_request_t *connect_connector_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
{
    list_t    *localVarQueryParameters = NULL;
    list_t    *localVarHeaderParameters = list_createList();
    list_t    *localVarFormParameters = NULL;
    list_t *localVarHeaderType = list_createList();
    list_t *localVarContentType = list_createList();
    char      *localVarBodyParameters = NULL;
    size_t     localVarBodyLength = 0;

    // clear the error code from the previous api call
    apiClient->response_code = 0;

    // create the path
    char *localVarPath = strdup("/v1/connectors/{tenant_id}/{project_id}/connect");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);



    // header parameters
    char *keyHeader_authorization = NULL;
    char * valueHeader_authorization = 0;
    keyValuePair_t *keyPairHeader_authorization = 0;
    if (authorization) {
        keyHeader_authorization = strdup("authorization");
        valueHeader_authorization = strdup((authorization));
        keyPairHeader_authorization = keyValuePair_create(keyHeader_authorization, valueHeader_authorization);
        list_addElement(localVarHeaderParameters,keyPairHeader_authorization);
    }


    // header parameters
    char *keyHeader_x_beater_api_key = NULL;
    char * valueHeader_x_beater_api_key = 0;
    keyValuePair_t *keyPairHeader_x_beater_api_key = 0;
    if (x_beater_api_key) {
        keyHeader_x_beater_api_key = strdup("x-beater-api-key");
        valueHeader_x_beater_api_key = strdup((x_beater_api_key));
        keyPairHeader_x_beater_api_key = keyValuePair_create(keyHeader_x_beater_api_key, valueHeader_x_beater_api_key);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_api_key);
    }


    // header parameters
    char *keyHeader_x_beater_project_id = NULL;
    char * valueHeader_x_beater_project_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_project_id = 0;
    if (x_beater_project_id) {
        keyHeader_x_beater_project_id = strdup("x-beater-project-id");
        valueHeader_x_beater_project_id = strdup((x_beater_project_id));
        keyPairHeader_x_beater_project_id = keyValuePair_create(keyHeader_x_beater_project_id, valueHeader_x_beater_project_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_project_id);
    }


    // header parameters
    char *keyHeader_x_beater_environment_id = NULL;
    char * valueHeader_x_beater_environment_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_environment_id = 0;
    if (x_beater_environment_id) {
        keyHeader_x_beater_environment_id = strdup("x-beater-environment-id");
        valueHeader_x_beater_environment_id = strdup((x_beater_environment_id));
        keyPairHeader_x_beater_environment_id = keyValuePair_create(keyHeader_x_beater_environment_id, valueHeader_x_beater_environment_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_environment_id);
    }


    // Body Param
    cJSON *localVarSingleItemJSON_connect_connector_request = NULL;
    if (connect_connector_request != NULL)
    {
        //not string, not binary
        localVarSingleItemJSON_connect_connector_request = connect_connector_request_convertToJSON(connect_connector_request);
        localVarBodyParameters = cJSON_Print(localVarSingleItemJSON_connect_connector_request);
        localVarBodyLength = strlen(localVarBodyParameters);
    }
    list_addElement(localVarHeaderType,"application/json"); //produces
    list_addElement(localVarContentType,"application/json"); //consumes
    apiClient_invoke(apiClient,
                    localVarPath,
                    localVarQueryParameters,
                    localVarHeaderParameters,
                    localVarFormParameters,
                    localVarHeaderType,
                    localVarContentType,
                    localVarBodyParameters,
                    localVarBodyLength,
                    "POST");

    // uncomment below to debug the error response
    //if (apiClient->response_code == 200) {
    //    printf("%s\n","One-time login link to authorize the app");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 400) {
    //    printf("%s\n","Invalid request, scope, or filter");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 401) {
    //    printf("%s\n","Missing or invalid credentials");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 403) {
    //    printf("%s\n","Credentials lack the required scope");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 501) {
    //    printf("%s\n","Connector provider not configured");
    //}
    //nonprimitive not container
    connection_link_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *ConnectorsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = connection_link_parseFromJSON(ConnectorsAPIlocalVarJSON);
        cJSON_Delete(ConnectorsAPIlocalVarJSON);
        if(elementToReturn == NULL) {
            // return 0;
        }
    }

    //return type
    if (apiClient->dataReceived) {
        free(apiClient->dataReceived);
        apiClient->dataReceived = NULL;
        apiClient->dataReceivedLen = 0;
    }
    
    list_freeList(localVarHeaderParameters);
    
    list_freeList(localVarHeaderType);
    list_freeList(localVarContentType);
    free(localVarPath);
    free(localVarToReplace_tenant_id);
    free(localVarToReplace_project_id);
    if (keyHeader_authorization) {
        free(keyHeader_authorization);
        keyHeader_authorization = NULL;
    }
    if (valueHeader_authorization) {
        free(valueHeader_authorization);
        valueHeader_authorization = NULL;
    }
    free(keyPairHeader_authorization);
    if (keyHeader_x_beater_api_key) {
        free(keyHeader_x_beater_api_key);
        keyHeader_x_beater_api_key = NULL;
    }
    if (valueHeader_x_beater_api_key) {
        free(valueHeader_x_beater_api_key);
        valueHeader_x_beater_api_key = NULL;
    }
    free(keyPairHeader_x_beater_api_key);
    if (keyHeader_x_beater_project_id) {
        free(keyHeader_x_beater_project_id);
        keyHeader_x_beater_project_id = NULL;
    }
    if (valueHeader_x_beater_project_id) {
        free(valueHeader_x_beater_project_id);
        valueHeader_x_beater_project_id = NULL;
    }
    free(keyPairHeader_x_beater_project_id);
    if (keyHeader_x_beater_environment_id) {
        free(keyHeader_x_beater_environment_id);
        keyHeader_x_beater_environment_id = NULL;
    }
    if (valueHeader_x_beater_environment_id) {
        free(valueHeader_x_beater_environment_id);
        valueHeader_x_beater_environment_id = NULL;
    }
    free(keyPairHeader_x_beater_environment_id);
    if (localVarSingleItemJSON_connect_connector_request) {
        cJSON_Delete(localVarSingleItemJSON_connect_connector_request);
        localVarSingleItemJSON_connect_connector_request = NULL;
    }
    free(localVarBodyParameters);
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

connection_status_t*
ConnectorsAPI_connectorStatus(apiClient_t *apiClient, char *tenant_id, char *project_id, char *toolkit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
{
    list_t    *localVarQueryParameters = list_createList();
    list_t    *localVarHeaderParameters = list_createList();
    list_t    *localVarFormParameters = NULL;
    list_t *localVarHeaderType = list_createList();
    list_t *localVarContentType = NULL;
    char      *localVarBodyParameters = NULL;
    size_t     localVarBodyLength = 0;

    // clear the error code from the previous api call
    apiClient->response_code = 0;

    // create the path
    char *localVarPath = strdup("/v1/connectors/{tenant_id}/{project_id}/status");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);



    // header parameters
    char *keyHeader_authorization = NULL;
    char * valueHeader_authorization = 0;
    keyValuePair_t *keyPairHeader_authorization = 0;
    if (authorization) {
        keyHeader_authorization = strdup("authorization");
        valueHeader_authorization = strdup((authorization));
        keyPairHeader_authorization = keyValuePair_create(keyHeader_authorization, valueHeader_authorization);
        list_addElement(localVarHeaderParameters,keyPairHeader_authorization);
    }


    // header parameters
    char *keyHeader_x_beater_api_key = NULL;
    char * valueHeader_x_beater_api_key = 0;
    keyValuePair_t *keyPairHeader_x_beater_api_key = 0;
    if (x_beater_api_key) {
        keyHeader_x_beater_api_key = strdup("x-beater-api-key");
        valueHeader_x_beater_api_key = strdup((x_beater_api_key));
        keyPairHeader_x_beater_api_key = keyValuePair_create(keyHeader_x_beater_api_key, valueHeader_x_beater_api_key);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_api_key);
    }


    // header parameters
    char *keyHeader_x_beater_project_id = NULL;
    char * valueHeader_x_beater_project_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_project_id = 0;
    if (x_beater_project_id) {
        keyHeader_x_beater_project_id = strdup("x-beater-project-id");
        valueHeader_x_beater_project_id = strdup((x_beater_project_id));
        keyPairHeader_x_beater_project_id = keyValuePair_create(keyHeader_x_beater_project_id, valueHeader_x_beater_project_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_project_id);
    }


    // header parameters
    char *keyHeader_x_beater_environment_id = NULL;
    char * valueHeader_x_beater_environment_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_environment_id = 0;
    if (x_beater_environment_id) {
        keyHeader_x_beater_environment_id = strdup("x-beater-environment-id");
        valueHeader_x_beater_environment_id = strdup((x_beater_environment_id));
        keyPairHeader_x_beater_environment_id = keyValuePair_create(keyHeader_x_beater_environment_id, valueHeader_x_beater_environment_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_environment_id);
    }


    // query parameters
    char *keyQuery_toolkit = NULL;
    char * valueQuery_toolkit = NULL;
    keyValuePair_t *keyPairQuery_toolkit = 0;
    if (toolkit)
    {
        keyQuery_toolkit = strdup("toolkit");
        valueQuery_toolkit = strdup((toolkit));
        keyPairQuery_toolkit = keyValuePair_create(keyQuery_toolkit, valueQuery_toolkit);
        list_addElement(localVarQueryParameters,keyPairQuery_toolkit);
    }
    list_addElement(localVarHeaderType,"application/json"); //produces
    apiClient_invoke(apiClient,
                    localVarPath,
                    localVarQueryParameters,
                    localVarHeaderParameters,
                    localVarFormParameters,
                    localVarHeaderType,
                    localVarContentType,
                    localVarBodyParameters,
                    localVarBodyLength,
                    "GET");

    // uncomment below to debug the error response
    //if (apiClient->response_code == 200) {
    //    printf("%s\n","Connection status of a toolkit for this project");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 400) {
    //    printf("%s\n","Invalid request, scope, or filter");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 401) {
    //    printf("%s\n","Missing or invalid credentials");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 403) {
    //    printf("%s\n","Credentials lack the required scope");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 501) {
    //    printf("%s\n","Connector provider not configured");
    //}
    //nonprimitive not container
    connection_status_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *ConnectorsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = connection_status_parseFromJSON(ConnectorsAPIlocalVarJSON);
        cJSON_Delete(ConnectorsAPIlocalVarJSON);
        if(elementToReturn == NULL) {
            // return 0;
        }
    }

    //return type
    if (apiClient->dataReceived) {
        free(apiClient->dataReceived);
        apiClient->dataReceived = NULL;
        apiClient->dataReceivedLen = 0;
    }
    list_freeList(localVarQueryParameters);
    list_freeList(localVarHeaderParameters);
    
    list_freeList(localVarHeaderType);
    
    free(localVarPath);
    free(localVarToReplace_tenant_id);
    free(localVarToReplace_project_id);
    if (keyHeader_authorization) {
        free(keyHeader_authorization);
        keyHeader_authorization = NULL;
    }
    if (valueHeader_authorization) {
        free(valueHeader_authorization);
        valueHeader_authorization = NULL;
    }
    free(keyPairHeader_authorization);
    if (keyHeader_x_beater_api_key) {
        free(keyHeader_x_beater_api_key);
        keyHeader_x_beater_api_key = NULL;
    }
    if (valueHeader_x_beater_api_key) {
        free(valueHeader_x_beater_api_key);
        valueHeader_x_beater_api_key = NULL;
    }
    free(keyPairHeader_x_beater_api_key);
    if (keyHeader_x_beater_project_id) {
        free(keyHeader_x_beater_project_id);
        keyHeader_x_beater_project_id = NULL;
    }
    if (valueHeader_x_beater_project_id) {
        free(valueHeader_x_beater_project_id);
        valueHeader_x_beater_project_id = NULL;
    }
    free(keyPairHeader_x_beater_project_id);
    if (keyHeader_x_beater_environment_id) {
        free(keyHeader_x_beater_environment_id);
        keyHeader_x_beater_environment_id = NULL;
    }
    if (valueHeader_x_beater_environment_id) {
        free(valueHeader_x_beater_environment_id);
        valueHeader_x_beater_environment_id = NULL;
    }
    free(keyPairHeader_x_beater_environment_id);
    if(keyQuery_toolkit){
        free(keyQuery_toolkit);
        keyQuery_toolkit = NULL;
    }
    if(valueQuery_toolkit){
        free(valueQuery_toolkit);
        valueQuery_toolkit = NULL;
    }
    if(keyPairQuery_toolkit){
        keyValuePair_free(keyPairQuery_toolkit);
        keyPairQuery_toolkit = NULL;
    }
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

connector_skills_response_t*
ConnectorsAPI_getConnectorSkills(apiClient_t *apiClient, char *tenant_id, char *project_id, char *toolkit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
{
    list_t    *localVarQueryParameters = list_createList();
    list_t    *localVarHeaderParameters = list_createList();
    list_t    *localVarFormParameters = NULL;
    list_t *localVarHeaderType = list_createList();
    list_t *localVarContentType = NULL;
    char      *localVarBodyParameters = NULL;
    size_t     localVarBodyLength = 0;

    // clear the error code from the previous api call
    apiClient->response_code = 0;

    // create the path
    char *localVarPath = strdup("/v1/connectors/{tenant_id}/{project_id}/skills");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);



    // header parameters
    char *keyHeader_authorization = NULL;
    char * valueHeader_authorization = 0;
    keyValuePair_t *keyPairHeader_authorization = 0;
    if (authorization) {
        keyHeader_authorization = strdup("authorization");
        valueHeader_authorization = strdup((authorization));
        keyPairHeader_authorization = keyValuePair_create(keyHeader_authorization, valueHeader_authorization);
        list_addElement(localVarHeaderParameters,keyPairHeader_authorization);
    }


    // header parameters
    char *keyHeader_x_beater_api_key = NULL;
    char * valueHeader_x_beater_api_key = 0;
    keyValuePair_t *keyPairHeader_x_beater_api_key = 0;
    if (x_beater_api_key) {
        keyHeader_x_beater_api_key = strdup("x-beater-api-key");
        valueHeader_x_beater_api_key = strdup((x_beater_api_key));
        keyPairHeader_x_beater_api_key = keyValuePair_create(keyHeader_x_beater_api_key, valueHeader_x_beater_api_key);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_api_key);
    }


    // header parameters
    char *keyHeader_x_beater_project_id = NULL;
    char * valueHeader_x_beater_project_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_project_id = 0;
    if (x_beater_project_id) {
        keyHeader_x_beater_project_id = strdup("x-beater-project-id");
        valueHeader_x_beater_project_id = strdup((x_beater_project_id));
        keyPairHeader_x_beater_project_id = keyValuePair_create(keyHeader_x_beater_project_id, valueHeader_x_beater_project_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_project_id);
    }


    // header parameters
    char *keyHeader_x_beater_environment_id = NULL;
    char * valueHeader_x_beater_environment_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_environment_id = 0;
    if (x_beater_environment_id) {
        keyHeader_x_beater_environment_id = strdup("x-beater-environment-id");
        valueHeader_x_beater_environment_id = strdup((x_beater_environment_id));
        keyPairHeader_x_beater_environment_id = keyValuePair_create(keyHeader_x_beater_environment_id, valueHeader_x_beater_environment_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_environment_id);
    }


    // query parameters
    char *keyQuery_toolkit = NULL;
    char * valueQuery_toolkit = NULL;
    keyValuePair_t *keyPairQuery_toolkit = 0;
    if (toolkit)
    {
        keyQuery_toolkit = strdup("toolkit");
        valueQuery_toolkit = strdup((toolkit));
        keyPairQuery_toolkit = keyValuePair_create(keyQuery_toolkit, valueQuery_toolkit);
        list_addElement(localVarQueryParameters,keyPairQuery_toolkit);
    }
    list_addElement(localVarHeaderType,"application/json"); //produces
    apiClient_invoke(apiClient,
                    localVarPath,
                    localVarQueryParameters,
                    localVarHeaderParameters,
                    localVarFormParameters,
                    localVarHeaderType,
                    localVarContentType,
                    localVarBodyParameters,
                    localVarBodyLength,
                    "GET");

    // uncomment below to debug the error response
    //if (apiClient->response_code == 200) {
    //    printf("%s\n","Generated prompting scaffold (skill cards) for a toolkit");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 400) {
    //    printf("%s\n","Invalid request, scope, or filter");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 401) {
    //    printf("%s\n","Missing or invalid credentials");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 403) {
    //    printf("%s\n","Credentials lack the required scope");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 501) {
    //    printf("%s\n","Connector provider not configured");
    //}
    //nonprimitive not container
    connector_skills_response_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *ConnectorsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = connector_skills_response_parseFromJSON(ConnectorsAPIlocalVarJSON);
        cJSON_Delete(ConnectorsAPIlocalVarJSON);
        if(elementToReturn == NULL) {
            // return 0;
        }
    }

    //return type
    if (apiClient->dataReceived) {
        free(apiClient->dataReceived);
        apiClient->dataReceived = NULL;
        apiClient->dataReceivedLen = 0;
    }
    list_freeList(localVarQueryParameters);
    list_freeList(localVarHeaderParameters);
    
    list_freeList(localVarHeaderType);
    
    free(localVarPath);
    free(localVarToReplace_tenant_id);
    free(localVarToReplace_project_id);
    if (keyHeader_authorization) {
        free(keyHeader_authorization);
        keyHeader_authorization = NULL;
    }
    if (valueHeader_authorization) {
        free(valueHeader_authorization);
        valueHeader_authorization = NULL;
    }
    free(keyPairHeader_authorization);
    if (keyHeader_x_beater_api_key) {
        free(keyHeader_x_beater_api_key);
        keyHeader_x_beater_api_key = NULL;
    }
    if (valueHeader_x_beater_api_key) {
        free(valueHeader_x_beater_api_key);
        valueHeader_x_beater_api_key = NULL;
    }
    free(keyPairHeader_x_beater_api_key);
    if (keyHeader_x_beater_project_id) {
        free(keyHeader_x_beater_project_id);
        keyHeader_x_beater_project_id = NULL;
    }
    if (valueHeader_x_beater_project_id) {
        free(valueHeader_x_beater_project_id);
        valueHeader_x_beater_project_id = NULL;
    }
    free(keyPairHeader_x_beater_project_id);
    if (keyHeader_x_beater_environment_id) {
        free(keyHeader_x_beater_environment_id);
        keyHeader_x_beater_environment_id = NULL;
    }
    if (valueHeader_x_beater_environment_id) {
        free(valueHeader_x_beater_environment_id);
        valueHeader_x_beater_environment_id = NULL;
    }
    free(keyPairHeader_x_beater_environment_id);
    if(keyQuery_toolkit){
        free(keyQuery_toolkit);
        keyQuery_toolkit = NULL;
    }
    if(valueQuery_toolkit){
        free(valueQuery_toolkit);
        valueQuery_toolkit = NULL;
    }
    if(keyPairQuery_toolkit){
        keyValuePair_free(keyPairQuery_toolkit);
        keyPairQuery_toolkit = NULL;
    }
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

tool_execution_t*
ConnectorsAPI_invokeConnectorTool(apiClient_t *apiClient, char *tenant_id, char *project_id, invoke_connector_request_t *invoke_connector_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
{
    list_t    *localVarQueryParameters = NULL;
    list_t    *localVarHeaderParameters = list_createList();
    list_t    *localVarFormParameters = NULL;
    list_t *localVarHeaderType = list_createList();
    list_t *localVarContentType = list_createList();
    char      *localVarBodyParameters = NULL;
    size_t     localVarBodyLength = 0;

    // clear the error code from the previous api call
    apiClient->response_code = 0;

    // create the path
    char *localVarPath = strdup("/v1/connectors/{tenant_id}/{project_id}/invoke");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);



    // header parameters
    char *keyHeader_authorization = NULL;
    char * valueHeader_authorization = 0;
    keyValuePair_t *keyPairHeader_authorization = 0;
    if (authorization) {
        keyHeader_authorization = strdup("authorization");
        valueHeader_authorization = strdup((authorization));
        keyPairHeader_authorization = keyValuePair_create(keyHeader_authorization, valueHeader_authorization);
        list_addElement(localVarHeaderParameters,keyPairHeader_authorization);
    }


    // header parameters
    char *keyHeader_x_beater_api_key = NULL;
    char * valueHeader_x_beater_api_key = 0;
    keyValuePair_t *keyPairHeader_x_beater_api_key = 0;
    if (x_beater_api_key) {
        keyHeader_x_beater_api_key = strdup("x-beater-api-key");
        valueHeader_x_beater_api_key = strdup((x_beater_api_key));
        keyPairHeader_x_beater_api_key = keyValuePair_create(keyHeader_x_beater_api_key, valueHeader_x_beater_api_key);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_api_key);
    }


    // header parameters
    char *keyHeader_x_beater_project_id = NULL;
    char * valueHeader_x_beater_project_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_project_id = 0;
    if (x_beater_project_id) {
        keyHeader_x_beater_project_id = strdup("x-beater-project-id");
        valueHeader_x_beater_project_id = strdup((x_beater_project_id));
        keyPairHeader_x_beater_project_id = keyValuePair_create(keyHeader_x_beater_project_id, valueHeader_x_beater_project_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_project_id);
    }


    // header parameters
    char *keyHeader_x_beater_environment_id = NULL;
    char * valueHeader_x_beater_environment_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_environment_id = 0;
    if (x_beater_environment_id) {
        keyHeader_x_beater_environment_id = strdup("x-beater-environment-id");
        valueHeader_x_beater_environment_id = strdup((x_beater_environment_id));
        keyPairHeader_x_beater_environment_id = keyValuePair_create(keyHeader_x_beater_environment_id, valueHeader_x_beater_environment_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_environment_id);
    }


    // Body Param
    cJSON *localVarSingleItemJSON_invoke_connector_request = NULL;
    if (invoke_connector_request != NULL)
    {
        //not string, not binary
        localVarSingleItemJSON_invoke_connector_request = invoke_connector_request_convertToJSON(invoke_connector_request);
        localVarBodyParameters = cJSON_Print(localVarSingleItemJSON_invoke_connector_request);
        localVarBodyLength = strlen(localVarBodyParameters);
    }
    list_addElement(localVarHeaderType,"application/json"); //produces
    list_addElement(localVarContentType,"application/json"); //consumes
    apiClient_invoke(apiClient,
                    localVarPath,
                    localVarQueryParameters,
                    localVarHeaderParameters,
                    localVarFormParameters,
                    localVarHeaderType,
                    localVarContentType,
                    localVarBodyParameters,
                    localVarBodyLength,
                    "POST");

    // uncomment below to debug the error response
    //if (apiClient->response_code == 200) {
    //    printf("%s\n","Execute a connector tool and return its result envelope");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 400) {
    //    printf("%s\n","Invalid request, scope, or filter");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 401) {
    //    printf("%s\n","Missing or invalid credentials");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 403) {
    //    printf("%s\n","Credentials lack the required scope");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 501) {
    //    printf("%s\n","Connector provider not configured");
    //}
    //nonprimitive not container
    tool_execution_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *ConnectorsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = tool_execution_parseFromJSON(ConnectorsAPIlocalVarJSON);
        cJSON_Delete(ConnectorsAPIlocalVarJSON);
        if(elementToReturn == NULL) {
            // return 0;
        }
    }

    //return type
    if (apiClient->dataReceived) {
        free(apiClient->dataReceived);
        apiClient->dataReceived = NULL;
        apiClient->dataReceivedLen = 0;
    }
    
    list_freeList(localVarHeaderParameters);
    
    list_freeList(localVarHeaderType);
    list_freeList(localVarContentType);
    free(localVarPath);
    free(localVarToReplace_tenant_id);
    free(localVarToReplace_project_id);
    if (keyHeader_authorization) {
        free(keyHeader_authorization);
        keyHeader_authorization = NULL;
    }
    if (valueHeader_authorization) {
        free(valueHeader_authorization);
        valueHeader_authorization = NULL;
    }
    free(keyPairHeader_authorization);
    if (keyHeader_x_beater_api_key) {
        free(keyHeader_x_beater_api_key);
        keyHeader_x_beater_api_key = NULL;
    }
    if (valueHeader_x_beater_api_key) {
        free(valueHeader_x_beater_api_key);
        valueHeader_x_beater_api_key = NULL;
    }
    free(keyPairHeader_x_beater_api_key);
    if (keyHeader_x_beater_project_id) {
        free(keyHeader_x_beater_project_id);
        keyHeader_x_beater_project_id = NULL;
    }
    if (valueHeader_x_beater_project_id) {
        free(valueHeader_x_beater_project_id);
        valueHeader_x_beater_project_id = NULL;
    }
    free(keyPairHeader_x_beater_project_id);
    if (keyHeader_x_beater_environment_id) {
        free(keyHeader_x_beater_environment_id);
        keyHeader_x_beater_environment_id = NULL;
    }
    if (valueHeader_x_beater_environment_id) {
        free(valueHeader_x_beater_environment_id);
        valueHeader_x_beater_environment_id = NULL;
    }
    free(keyPairHeader_x_beater_environment_id);
    if (localVarSingleItemJSON_invoke_connector_request) {
        cJSON_Delete(localVarSingleItemJSON_invoke_connector_request);
        localVarSingleItemJSON_invoke_connector_request = NULL;
    }
    free(localVarBodyParameters);
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

list_t*
ConnectorsAPI_listConnectorTools(apiClient_t *apiClient, char *tenant_id, char *project_id, char *toolkit, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
{
    list_t    *localVarQueryParameters = list_createList();
    list_t    *localVarHeaderParameters = list_createList();
    list_t    *localVarFormParameters = NULL;
    list_t *localVarHeaderType = list_createList();
    list_t *localVarContentType = NULL;
    char      *localVarBodyParameters = NULL;
    size_t     localVarBodyLength = 0;

    // clear the error code from the previous api call
    apiClient->response_code = 0;

    // create the path
    char *localVarPath = strdup("/v1/connectors/{tenant_id}/{project_id}/tools");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);



    // header parameters
    char *keyHeader_authorization = NULL;
    char * valueHeader_authorization = 0;
    keyValuePair_t *keyPairHeader_authorization = 0;
    if (authorization) {
        keyHeader_authorization = strdup("authorization");
        valueHeader_authorization = strdup((authorization));
        keyPairHeader_authorization = keyValuePair_create(keyHeader_authorization, valueHeader_authorization);
        list_addElement(localVarHeaderParameters,keyPairHeader_authorization);
    }


    // header parameters
    char *keyHeader_x_beater_api_key = NULL;
    char * valueHeader_x_beater_api_key = 0;
    keyValuePair_t *keyPairHeader_x_beater_api_key = 0;
    if (x_beater_api_key) {
        keyHeader_x_beater_api_key = strdup("x-beater-api-key");
        valueHeader_x_beater_api_key = strdup((x_beater_api_key));
        keyPairHeader_x_beater_api_key = keyValuePair_create(keyHeader_x_beater_api_key, valueHeader_x_beater_api_key);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_api_key);
    }


    // header parameters
    char *keyHeader_x_beater_project_id = NULL;
    char * valueHeader_x_beater_project_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_project_id = 0;
    if (x_beater_project_id) {
        keyHeader_x_beater_project_id = strdup("x-beater-project-id");
        valueHeader_x_beater_project_id = strdup((x_beater_project_id));
        keyPairHeader_x_beater_project_id = keyValuePair_create(keyHeader_x_beater_project_id, valueHeader_x_beater_project_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_project_id);
    }


    // header parameters
    char *keyHeader_x_beater_environment_id = NULL;
    char * valueHeader_x_beater_environment_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_environment_id = 0;
    if (x_beater_environment_id) {
        keyHeader_x_beater_environment_id = strdup("x-beater-environment-id");
        valueHeader_x_beater_environment_id = strdup((x_beater_environment_id));
        keyPairHeader_x_beater_environment_id = keyValuePair_create(keyHeader_x_beater_environment_id, valueHeader_x_beater_environment_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_environment_id);
    }


    // query parameters
    char *keyQuery_toolkit = NULL;
    char * valueQuery_toolkit = NULL;
    keyValuePair_t *keyPairQuery_toolkit = 0;
    if (toolkit)
    {
        keyQuery_toolkit = strdup("toolkit");
        valueQuery_toolkit = strdup((toolkit));
        keyPairQuery_toolkit = keyValuePair_create(keyQuery_toolkit, valueQuery_toolkit);
        list_addElement(localVarQueryParameters,keyPairQuery_toolkit);
    }

    // query parameters
    char *keyQuery_limit = NULL;
    char * valueQuery_limit = NULL;
    keyValuePair_t *keyPairQuery_limit = 0;
    if (limit)
    {
        keyQuery_limit = strdup("limit");
        valueQuery_limit = calloc(1,MAX_NUMBER_LENGTH);
        snprintf(valueQuery_limit, MAX_NUMBER_LENGTH, "%d", *limit);
        keyPairQuery_limit = keyValuePair_create(keyQuery_limit, valueQuery_limit);
        list_addElement(localVarQueryParameters,keyPairQuery_limit);
    }
    list_addElement(localVarHeaderType,"application/json"); //produces
    apiClient_invoke(apiClient,
                    localVarPath,
                    localVarQueryParameters,
                    localVarHeaderParameters,
                    localVarFormParameters,
                    localVarHeaderType,
                    localVarContentType,
                    localVarBodyParameters,
                    localVarBodyLength,
                    "GET");

    // uncomment below to debug the error response
    //if (apiClient->response_code == 200) {
    //    printf("%s\n","List a toolkit&#39;s executable tools with input schemas");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 400) {
    //    printf("%s\n","Invalid request, scope, or filter");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 401) {
    //    printf("%s\n","Missing or invalid credentials");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 403) {
    //    printf("%s\n","Credentials lack the required scope");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 501) {
    //    printf("%s\n","Connector provider not configured");
    //}
    list_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *ConnectorsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        if(!cJSON_IsArray(ConnectorsAPIlocalVarJSON)) {
            return 0;//nonprimitive container
        }
        elementToReturn = list_createList();
        cJSON *VarJSON;
        cJSON_ArrayForEach(VarJSON, ConnectorsAPIlocalVarJSON)
        {
            if(!cJSON_IsObject(VarJSON))
            {
               // return 0;
            }
            char *localVarJSONToChar = cJSON_Print(VarJSON);
            list_addElement(elementToReturn , localVarJSONToChar);
        }

        cJSON_Delete( ConnectorsAPIlocalVarJSON);
        cJSON_Delete( VarJSON);
    }
    //return type
    if (apiClient->dataReceived) {
        free(apiClient->dataReceived);
        apiClient->dataReceived = NULL;
        apiClient->dataReceivedLen = 0;
    }
    list_freeList(localVarQueryParameters);
    list_freeList(localVarHeaderParameters);
    
    list_freeList(localVarHeaderType);
    
    free(localVarPath);
    free(localVarToReplace_tenant_id);
    free(localVarToReplace_project_id);
    if (keyHeader_authorization) {
        free(keyHeader_authorization);
        keyHeader_authorization = NULL;
    }
    if (valueHeader_authorization) {
        free(valueHeader_authorization);
        valueHeader_authorization = NULL;
    }
    free(keyPairHeader_authorization);
    if (keyHeader_x_beater_api_key) {
        free(keyHeader_x_beater_api_key);
        keyHeader_x_beater_api_key = NULL;
    }
    if (valueHeader_x_beater_api_key) {
        free(valueHeader_x_beater_api_key);
        valueHeader_x_beater_api_key = NULL;
    }
    free(keyPairHeader_x_beater_api_key);
    if (keyHeader_x_beater_project_id) {
        free(keyHeader_x_beater_project_id);
        keyHeader_x_beater_project_id = NULL;
    }
    if (valueHeader_x_beater_project_id) {
        free(valueHeader_x_beater_project_id);
        valueHeader_x_beater_project_id = NULL;
    }
    free(keyPairHeader_x_beater_project_id);
    if (keyHeader_x_beater_environment_id) {
        free(keyHeader_x_beater_environment_id);
        keyHeader_x_beater_environment_id = NULL;
    }
    if (valueHeader_x_beater_environment_id) {
        free(valueHeader_x_beater_environment_id);
        valueHeader_x_beater_environment_id = NULL;
    }
    free(keyPairHeader_x_beater_environment_id);
    if(keyQuery_toolkit){
        free(keyQuery_toolkit);
        keyQuery_toolkit = NULL;
    }
    if(valueQuery_toolkit){
        free(valueQuery_toolkit);
        valueQuery_toolkit = NULL;
    }
    if(keyPairQuery_toolkit){
        keyValuePair_free(keyPairQuery_toolkit);
        keyPairQuery_toolkit = NULL;
    }
    if(keyQuery_limit){
        free(keyQuery_limit);
        keyQuery_limit = NULL;
    }
    if(valueQuery_limit){
        free(valueQuery_limit);
        valueQuery_limit = NULL;
    }
    if(keyPairQuery_limit){
        keyValuePair_free(keyPairQuery_limit);
        keyPairQuery_limit = NULL;
    }
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

list_t*
ConnectorsAPI_listConnectors(apiClient_t *apiClient, char *tenant_id, char *project_id, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
{
    list_t    *localVarQueryParameters = list_createList();
    list_t    *localVarHeaderParameters = list_createList();
    list_t    *localVarFormParameters = NULL;
    list_t *localVarHeaderType = list_createList();
    list_t *localVarContentType = NULL;
    char      *localVarBodyParameters = NULL;
    size_t     localVarBodyLength = 0;

    // clear the error code from the previous api call
    apiClient->response_code = 0;

    // create the path
    char *localVarPath = strdup("/v1/connectors/{tenant_id}/{project_id}");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);



    // header parameters
    char *keyHeader_authorization = NULL;
    char * valueHeader_authorization = 0;
    keyValuePair_t *keyPairHeader_authorization = 0;
    if (authorization) {
        keyHeader_authorization = strdup("authorization");
        valueHeader_authorization = strdup((authorization));
        keyPairHeader_authorization = keyValuePair_create(keyHeader_authorization, valueHeader_authorization);
        list_addElement(localVarHeaderParameters,keyPairHeader_authorization);
    }


    // header parameters
    char *keyHeader_x_beater_api_key = NULL;
    char * valueHeader_x_beater_api_key = 0;
    keyValuePair_t *keyPairHeader_x_beater_api_key = 0;
    if (x_beater_api_key) {
        keyHeader_x_beater_api_key = strdup("x-beater-api-key");
        valueHeader_x_beater_api_key = strdup((x_beater_api_key));
        keyPairHeader_x_beater_api_key = keyValuePair_create(keyHeader_x_beater_api_key, valueHeader_x_beater_api_key);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_api_key);
    }


    // header parameters
    char *keyHeader_x_beater_project_id = NULL;
    char * valueHeader_x_beater_project_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_project_id = 0;
    if (x_beater_project_id) {
        keyHeader_x_beater_project_id = strdup("x-beater-project-id");
        valueHeader_x_beater_project_id = strdup((x_beater_project_id));
        keyPairHeader_x_beater_project_id = keyValuePair_create(keyHeader_x_beater_project_id, valueHeader_x_beater_project_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_project_id);
    }


    // header parameters
    char *keyHeader_x_beater_environment_id = NULL;
    char * valueHeader_x_beater_environment_id = 0;
    keyValuePair_t *keyPairHeader_x_beater_environment_id = 0;
    if (x_beater_environment_id) {
        keyHeader_x_beater_environment_id = strdup("x-beater-environment-id");
        valueHeader_x_beater_environment_id = strdup((x_beater_environment_id));
        keyPairHeader_x_beater_environment_id = keyValuePair_create(keyHeader_x_beater_environment_id, valueHeader_x_beater_environment_id);
        list_addElement(localVarHeaderParameters,keyPairHeader_x_beater_environment_id);
    }


    // query parameters
    char *keyQuery_limit = NULL;
    char * valueQuery_limit = NULL;
    keyValuePair_t *keyPairQuery_limit = 0;
    if (limit)
    {
        keyQuery_limit = strdup("limit");
        valueQuery_limit = calloc(1,MAX_NUMBER_LENGTH);
        snprintf(valueQuery_limit, MAX_NUMBER_LENGTH, "%d", *limit);
        keyPairQuery_limit = keyValuePair_create(keyQuery_limit, valueQuery_limit);
        list_addElement(localVarQueryParameters,keyPairQuery_limit);
    }
    list_addElement(localVarHeaderType,"application/json"); //produces
    apiClient_invoke(apiClient,
                    localVarPath,
                    localVarQueryParameters,
                    localVarHeaderParameters,
                    localVarFormParameters,
                    localVarHeaderType,
                    localVarContentType,
                    localVarBodyParameters,
                    localVarBodyLength,
                    "GET");

    // uncomment below to debug the error response
    //if (apiClient->response_code == 200) {
    //    printf("%s\n","List connectable third-party apps (catalog)");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 400) {
    //    printf("%s\n","Invalid request, scope, or filter");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 401) {
    //    printf("%s\n","Missing or invalid credentials");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 403) {
    //    printf("%s\n","Credentials lack the required scope");
    //}
    // uncomment below to debug the error response
    //if (apiClient->response_code == 501) {
    //    printf("%s\n","Connector provider not configured");
    //}
    list_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *ConnectorsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        if(!cJSON_IsArray(ConnectorsAPIlocalVarJSON)) {
            return 0;//nonprimitive container
        }
        elementToReturn = list_createList();
        cJSON *VarJSON;
        cJSON_ArrayForEach(VarJSON, ConnectorsAPIlocalVarJSON)
        {
            if(!cJSON_IsObject(VarJSON))
            {
               // return 0;
            }
            char *localVarJSONToChar = cJSON_Print(VarJSON);
            list_addElement(elementToReturn , localVarJSONToChar);
        }

        cJSON_Delete( ConnectorsAPIlocalVarJSON);
        cJSON_Delete( VarJSON);
    }
    //return type
    if (apiClient->dataReceived) {
        free(apiClient->dataReceived);
        apiClient->dataReceived = NULL;
        apiClient->dataReceivedLen = 0;
    }
    list_freeList(localVarQueryParameters);
    list_freeList(localVarHeaderParameters);
    
    list_freeList(localVarHeaderType);
    
    free(localVarPath);
    free(localVarToReplace_tenant_id);
    free(localVarToReplace_project_id);
    if (keyHeader_authorization) {
        free(keyHeader_authorization);
        keyHeader_authorization = NULL;
    }
    if (valueHeader_authorization) {
        free(valueHeader_authorization);
        valueHeader_authorization = NULL;
    }
    free(keyPairHeader_authorization);
    if (keyHeader_x_beater_api_key) {
        free(keyHeader_x_beater_api_key);
        keyHeader_x_beater_api_key = NULL;
    }
    if (valueHeader_x_beater_api_key) {
        free(valueHeader_x_beater_api_key);
        valueHeader_x_beater_api_key = NULL;
    }
    free(keyPairHeader_x_beater_api_key);
    if (keyHeader_x_beater_project_id) {
        free(keyHeader_x_beater_project_id);
        keyHeader_x_beater_project_id = NULL;
    }
    if (valueHeader_x_beater_project_id) {
        free(valueHeader_x_beater_project_id);
        valueHeader_x_beater_project_id = NULL;
    }
    free(keyPairHeader_x_beater_project_id);
    if (keyHeader_x_beater_environment_id) {
        free(keyHeader_x_beater_environment_id);
        keyHeader_x_beater_environment_id = NULL;
    }
    if (valueHeader_x_beater_environment_id) {
        free(valueHeader_x_beater_environment_id);
        valueHeader_x_beater_environment_id = NULL;
    }
    free(keyPairHeader_x_beater_environment_id);
    if(keyQuery_limit){
        free(keyQuery_limit);
        keyQuery_limit = NULL;
    }
    if(valueQuery_limit){
        free(valueQuery_limit);
        valueQuery_limit = NULL;
    }
    if(keyPairQuery_limit){
        keyValuePair_free(keyPairQuery_limit);
        keyPairQuery_limit = NULL;
    }
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

