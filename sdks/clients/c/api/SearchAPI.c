#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>
#include "SearchAPI.h"

#define MAX_NUMBER_LENGTH 16
#define MAX_BUFFER_LENGTH 4096


search_response_t*
SearchAPI_searchSearchSpans(apiClient_t *apiClient, char *tenant_id, char *q, char *project_id, char *environment_id, char *trace_id, char *span_id, char *kind, char *status, char *model, char *tool, int *limit, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
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
    char *localVarPath = strdup("/v1/search/{tenant_id}/spans");

    if(!tenant_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);



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
    char *keyQuery_q = NULL;
    char * valueQuery_q = NULL;
    keyValuePair_t *keyPairQuery_q = 0;
    if (q)
    {
        keyQuery_q = strdup("q");
        valueQuery_q = strdup((q));
        keyPairQuery_q = keyValuePair_create(keyQuery_q, valueQuery_q);
        list_addElement(localVarQueryParameters,keyPairQuery_q);
    }

    // query parameters
    char *keyQuery_project_id = NULL;
    char * valueQuery_project_id = NULL;
    keyValuePair_t *keyPairQuery_project_id = 0;
    if (project_id)
    {
        keyQuery_project_id = strdup("project_id");
        valueQuery_project_id = strdup((project_id));
        keyPairQuery_project_id = keyValuePair_create(keyQuery_project_id, valueQuery_project_id);
        list_addElement(localVarQueryParameters,keyPairQuery_project_id);
    }

    // query parameters
    char *keyQuery_environment_id = NULL;
    char * valueQuery_environment_id = NULL;
    keyValuePair_t *keyPairQuery_environment_id = 0;
    if (environment_id)
    {
        keyQuery_environment_id = strdup("environment_id");
        valueQuery_environment_id = strdup((environment_id));
        keyPairQuery_environment_id = keyValuePair_create(keyQuery_environment_id, valueQuery_environment_id);
        list_addElement(localVarQueryParameters,keyPairQuery_environment_id);
    }

    // query parameters
    char *keyQuery_trace_id = NULL;
    char * valueQuery_trace_id = NULL;
    keyValuePair_t *keyPairQuery_trace_id = 0;
    if (trace_id)
    {
        keyQuery_trace_id = strdup("trace_id");
        valueQuery_trace_id = strdup((trace_id));
        keyPairQuery_trace_id = keyValuePair_create(keyQuery_trace_id, valueQuery_trace_id);
        list_addElement(localVarQueryParameters,keyPairQuery_trace_id);
    }

    // query parameters
    char *keyQuery_span_id = NULL;
    char * valueQuery_span_id = NULL;
    keyValuePair_t *keyPairQuery_span_id = 0;
    if (span_id)
    {
        keyQuery_span_id = strdup("span_id");
        valueQuery_span_id = strdup((span_id));
        keyPairQuery_span_id = keyValuePair_create(keyQuery_span_id, valueQuery_span_id);
        list_addElement(localVarQueryParameters,keyPairQuery_span_id);
    }

    // query parameters
    char *keyQuery_kind = NULL;
    char * valueQuery_kind = NULL;
    keyValuePair_t *keyPairQuery_kind = 0;
    if (kind)
    {
        keyQuery_kind = strdup("kind");
        valueQuery_kind = strdup((kind));
        keyPairQuery_kind = keyValuePair_create(keyQuery_kind, valueQuery_kind);
        list_addElement(localVarQueryParameters,keyPairQuery_kind);
    }

    // query parameters
    char *keyQuery_status = NULL;
    char * valueQuery_status = NULL;
    keyValuePair_t *keyPairQuery_status = 0;
    if (status)
    {
        keyQuery_status = strdup("status");
        valueQuery_status = strdup((status));
        keyPairQuery_status = keyValuePair_create(keyQuery_status, valueQuery_status);
        list_addElement(localVarQueryParameters,keyPairQuery_status);
    }

    // query parameters
    char *keyQuery_model = NULL;
    char * valueQuery_model = NULL;
    keyValuePair_t *keyPairQuery_model = 0;
    if (model)
    {
        keyQuery_model = strdup("model");
        valueQuery_model = strdup((model));
        keyPairQuery_model = keyValuePair_create(keyQuery_model, valueQuery_model);
        list_addElement(localVarQueryParameters,keyPairQuery_model);
    }

    // query parameters
    char *keyQuery_tool = NULL;
    char * valueQuery_tool = NULL;
    keyValuePair_t *keyPairQuery_tool = 0;
    if (tool)
    {
        keyQuery_tool = strdup("tool");
        valueQuery_tool = strdup((tool));
        keyPairQuery_tool = keyValuePair_create(keyQuery_tool, valueQuery_tool);
        list_addElement(localVarQueryParameters,keyPairQuery_tool);
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
    //    printf("%s\n","Search spans");
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
    search_response_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *SearchAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = search_response_parseFromJSON(SearchAPIlocalVarJSON);
        cJSON_Delete(SearchAPIlocalVarJSON);
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
    if(keyQuery_q){
        free(keyQuery_q);
        keyQuery_q = NULL;
    }
    if(valueQuery_q){
        free(valueQuery_q);
        valueQuery_q = NULL;
    }
    if(keyPairQuery_q){
        keyValuePair_free(keyPairQuery_q);
        keyPairQuery_q = NULL;
    }
    if(keyQuery_project_id){
        free(keyQuery_project_id);
        keyQuery_project_id = NULL;
    }
    if(valueQuery_project_id){
        free(valueQuery_project_id);
        valueQuery_project_id = NULL;
    }
    if(keyPairQuery_project_id){
        keyValuePair_free(keyPairQuery_project_id);
        keyPairQuery_project_id = NULL;
    }
    if(keyQuery_environment_id){
        free(keyQuery_environment_id);
        keyQuery_environment_id = NULL;
    }
    if(valueQuery_environment_id){
        free(valueQuery_environment_id);
        valueQuery_environment_id = NULL;
    }
    if(keyPairQuery_environment_id){
        keyValuePair_free(keyPairQuery_environment_id);
        keyPairQuery_environment_id = NULL;
    }
    if(keyQuery_trace_id){
        free(keyQuery_trace_id);
        keyQuery_trace_id = NULL;
    }
    if(valueQuery_trace_id){
        free(valueQuery_trace_id);
        valueQuery_trace_id = NULL;
    }
    if(keyPairQuery_trace_id){
        keyValuePair_free(keyPairQuery_trace_id);
        keyPairQuery_trace_id = NULL;
    }
    if(keyQuery_span_id){
        free(keyQuery_span_id);
        keyQuery_span_id = NULL;
    }
    if(valueQuery_span_id){
        free(valueQuery_span_id);
        valueQuery_span_id = NULL;
    }
    if(keyPairQuery_span_id){
        keyValuePair_free(keyPairQuery_span_id);
        keyPairQuery_span_id = NULL;
    }
    if(keyQuery_kind){
        free(keyQuery_kind);
        keyQuery_kind = NULL;
    }
    if(valueQuery_kind){
        free(valueQuery_kind);
        valueQuery_kind = NULL;
    }
    if(keyPairQuery_kind){
        keyValuePair_free(keyPairQuery_kind);
        keyPairQuery_kind = NULL;
    }
    if(keyQuery_status){
        free(keyQuery_status);
        keyQuery_status = NULL;
    }
    if(valueQuery_status){
        free(valueQuery_status);
        valueQuery_status = NULL;
    }
    if(keyPairQuery_status){
        keyValuePair_free(keyPairQuery_status);
        keyPairQuery_status = NULL;
    }
    if(keyQuery_model){
        free(keyQuery_model);
        keyQuery_model = NULL;
    }
    if(valueQuery_model){
        free(valueQuery_model);
        valueQuery_model = NULL;
    }
    if(keyPairQuery_model){
        keyValuePair_free(keyPairQuery_model);
        keyPairQuery_model = NULL;
    }
    if(keyQuery_tool){
        free(keyQuery_tool);
        keyQuery_tool = NULL;
    }
    if(valueQuery_tool){
        free(valueQuery_tool);
        valueQuery_tool = NULL;
    }
    if(keyPairQuery_tool){
        keyValuePair_free(keyPairQuery_tool);
        keyPairQuery_tool = NULL;
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

