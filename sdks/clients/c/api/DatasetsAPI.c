#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>
#include "DatasetsAPI.h"

#define MAX_NUMBER_LENGTH 16
#define MAX_BUFFER_LENGTH 4096


dataset_t*
DatasetsAPI_datasetsCreateDataset(apiClient_t *apiClient, char *tenant_id, char *project_id, create_dataset_request_t *create_dataset_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
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
    char *localVarPath = strdup("/v1/datasets/{tenant_id}/{project_id}");

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
    cJSON *localVarSingleItemJSON_create_dataset_request = NULL;
    if (create_dataset_request != NULL)
    {
        //not string, not binary
        localVarSingleItemJSON_create_dataset_request = create_dataset_request_convertToJSON(create_dataset_request);
        localVarBodyParameters = cJSON_Print(localVarSingleItemJSON_create_dataset_request);
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
    //    printf("%s\n","Create a dataset");
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
    //nonprimitive not container
    dataset_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *DatasetsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = dataset_parseFromJSON(DatasetsAPIlocalVarJSON);
        cJSON_Delete(DatasetsAPIlocalVarJSON);
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
    if (localVarSingleItemJSON_create_dataset_request) {
        cJSON_Delete(localVarSingleItemJSON_create_dataset_request);
        localVarSingleItemJSON_create_dataset_request = NULL;
    }
    free(localVarBodyParameters);
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

dataset_version_snapshot_t*
DatasetsAPI_datasetsCreateDatasetVersion(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, create_dataset_version_request_t *create_dataset_version_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
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
    char *localVarPath = strdup("/v1/datasets/{tenant_id}/{project_id}/{dataset_id}/versions");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;
    if(!dataset_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + strlen(dataset_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + strlen(dataset_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);

    // Path Params
    long sizeOfPathParams_dataset_id = strlen(tenant_id)+3 + strlen(project_id)+3 + strlen(dataset_id)+3 + sizeof("{ dataset_id }") - 1;
    if(dataset_id == NULL) {
        goto end;
    }
    char* localVarToReplace_dataset_id = malloc(sizeOfPathParams_dataset_id);
    sprintf(localVarToReplace_dataset_id, "{%s}", "dataset_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_dataset_id, dataset_id);



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
    cJSON *localVarSingleItemJSON_create_dataset_version_request = NULL;
    if (create_dataset_version_request != NULL)
    {
        //not string, not binary
        localVarSingleItemJSON_create_dataset_version_request = create_dataset_version_request_convertToJSON(create_dataset_version_request);
        localVarBodyParameters = cJSON_Print(localVarSingleItemJSON_create_dataset_version_request);
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
    //    printf("%s\n","Create a dataset version snapshot");
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
    //if (apiClient->response_code == 404) {
    //    printf("%s\n","Resource not found");
    //}
    //nonprimitive not container
    dataset_version_snapshot_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *DatasetsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = dataset_version_snapshot_parseFromJSON(DatasetsAPIlocalVarJSON);
        cJSON_Delete(DatasetsAPIlocalVarJSON);
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
    free(localVarToReplace_dataset_id);
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
    if (localVarSingleItemJSON_create_dataset_version_request) {
        cJSON_Delete(localVarSingleItemJSON_create_dataset_version_request);
        localVarSingleItemJSON_create_dataset_version_request = NULL;
    }
    free(localVarBodyParameters);
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

dataset_case_t*
DatasetsAPI_datasetsPromoteDatasetCaseFromTrace(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, promote_trace_case_request_t *promote_trace_case_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
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
    char *localVarPath = strdup("/v1/datasets/{tenant_id}/{project_id}/{dataset_id}/cases/from-trace");

    if(!tenant_id)
        goto end;
    if(!project_id)
        goto end;
    if(!dataset_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(project_id)+3 + strlen(dataset_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_project_id = strlen(tenant_id)+3 + strlen(project_id)+3 + strlen(dataset_id)+3 + sizeof("{ project_id }") - 1;
    if(project_id == NULL) {
        goto end;
    }
    char* localVarToReplace_project_id = malloc(sizeOfPathParams_project_id);
    sprintf(localVarToReplace_project_id, "{%s}", "project_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_project_id, project_id);

    // Path Params
    long sizeOfPathParams_dataset_id = strlen(tenant_id)+3 + strlen(project_id)+3 + strlen(dataset_id)+3 + sizeof("{ dataset_id }") - 1;
    if(dataset_id == NULL) {
        goto end;
    }
    char* localVarToReplace_dataset_id = malloc(sizeOfPathParams_dataset_id);
    sprintf(localVarToReplace_dataset_id, "{%s}", "dataset_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_dataset_id, dataset_id);



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
    cJSON *localVarSingleItemJSON_promote_trace_case_request = NULL;
    if (promote_trace_case_request != NULL)
    {
        //not string, not binary
        localVarSingleItemJSON_promote_trace_case_request = promote_trace_case_request_convertToJSON(promote_trace_case_request);
        localVarBodyParameters = cJSON_Print(localVarSingleItemJSON_promote_trace_case_request);
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
    //    printf("%s\n","Promote a trace span to a dataset case");
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
    //if (apiClient->response_code == 404) {
    //    printf("%s\n","Resource not found");
    //}
    //nonprimitive not container
    dataset_case_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *DatasetsAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = dataset_case_parseFromJSON(DatasetsAPIlocalVarJSON);
        cJSON_Delete(DatasetsAPIlocalVarJSON);
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
    free(localVarToReplace_dataset_id);
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
    if (localVarSingleItemJSON_promote_trace_case_request) {
        cJSON_Delete(localVarSingleItemJSON_promote_trace_case_request);
        localVarSingleItemJSON_promote_trace_case_request = NULL;
    }
    free(localVarBodyParameters);
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

