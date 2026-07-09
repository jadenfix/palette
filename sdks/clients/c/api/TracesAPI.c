#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>
#include "TracesAPI.h"

#define MAX_NUMBER_LENGTH 16
#define MAX_BUFFER_LENGTH 4096


trace_view_t*
TracesAPI_tracesGetTrace(apiClient_t *apiClient, char *tenant_id, char *trace_id, int *unmask, char *reason, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
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
    char *localVarPath = strdup("/v1/traces/{tenant_id}/{trace_id}");

    if(!tenant_id)
        goto end;
    if(!trace_id)
        goto end;


    // Path Params
    long sizeOfPathParams_tenant_id = strlen(tenant_id)+3 + strlen(trace_id)+3 + sizeof("{ tenant_id }") - 1;
    if(tenant_id == NULL) {
        goto end;
    }
    char* localVarToReplace_tenant_id = malloc(sizeOfPathParams_tenant_id);
    sprintf(localVarToReplace_tenant_id, "{%s}", "tenant_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_tenant_id, tenant_id);

    // Path Params
    long sizeOfPathParams_trace_id = strlen(tenant_id)+3 + strlen(trace_id)+3 + sizeof("{ trace_id }") - 1;
    if(trace_id == NULL) {
        goto end;
    }
    char* localVarToReplace_trace_id = malloc(sizeOfPathParams_trace_id);
    sprintf(localVarToReplace_trace_id, "{%s}", "trace_id");

    localVarPath = strReplace(localVarPath, localVarToReplace_trace_id, trace_id);



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
    char *keyQuery_unmask = NULL;
    char * valueQuery_unmask = NULL;
    keyValuePair_t *keyPairQuery_unmask = 0;
    if (unmask)
    {
        keyQuery_unmask = strdup("unmask");
        valueQuery_unmask = calloc(1,MAX_NUMBER_LENGTH);
        snprintf(valueQuery_unmask, MAX_NUMBER_LENGTH, "%d", *unmask);
        keyPairQuery_unmask = keyValuePair_create(keyQuery_unmask, valueQuery_unmask);
        list_addElement(localVarQueryParameters,keyPairQuery_unmask);
    }

    // query parameters
    char *keyQuery_reason = NULL;
    char * valueQuery_reason = NULL;
    keyValuePair_t *keyPairQuery_reason = 0;
    if (reason)
    {
        keyQuery_reason = strdup("reason");
        valueQuery_reason = strdup((reason));
        keyPairQuery_reason = keyValuePair_create(keyQuery_reason, valueQuery_reason);
        list_addElement(localVarQueryParameters,keyPairQuery_reason);
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
    //    printf("%s\n","Get a canonical trace");
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
    trace_view_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *TracesAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = trace_view_parseFromJSON(TracesAPIlocalVarJSON);
        cJSON_Delete(TracesAPIlocalVarJSON);
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
    free(localVarToReplace_trace_id);
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
    if(keyQuery_unmask){
        free(keyQuery_unmask);
        keyQuery_unmask = NULL;
    }
    if(valueQuery_unmask){
        free(valueQuery_unmask);
        valueQuery_unmask = NULL;
    }
    if(keyPairQuery_unmask){
        keyValuePair_free(keyPairQuery_unmask);
        keyPairQuery_unmask = NULL;
    }
    if(keyQuery_reason){
        free(keyQuery_reason);
        keyQuery_reason = NULL;
    }
    if(valueQuery_reason){
        free(valueQuery_reason);
        valueQuery_reason = NULL;
    }
    if(keyPairQuery_reason){
        keyValuePair_free(keyPairQuery_reason);
        keyPairQuery_reason = NULL;
    }
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

page_run_summary_t*
TracesAPI_tracesListTraces(apiClient_t *apiClient, char *tenant_id, char *project_id, char *environment_id, char *trace_id, char *kind, char *status, char *started_after, char *started_before, char *model, char *release, long min_cost_micros, long max_cost_micros, long min_latency_ms, long max_latency_ms, int *limit, char *cursor, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id)
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
    char *localVarPath = strdup("/v1/traces/{tenant_id}");

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
    char *keyQuery_started_after = NULL;
    char * valueQuery_started_after = NULL;
    keyValuePair_t *keyPairQuery_started_after = 0;
    if (started_after)
    {
        keyQuery_started_after = strdup("started_after");
        valueQuery_started_after = strdup((started_after));
        keyPairQuery_started_after = keyValuePair_create(keyQuery_started_after, valueQuery_started_after);
        list_addElement(localVarQueryParameters,keyPairQuery_started_after);
    }

    // query parameters
    char *keyQuery_started_before = NULL;
    char * valueQuery_started_before = NULL;
    keyValuePair_t *keyPairQuery_started_before = 0;
    if (started_before)
    {
        keyQuery_started_before = strdup("started_before");
        valueQuery_started_before = strdup((started_before));
        keyPairQuery_started_before = keyValuePair_create(keyQuery_started_before, valueQuery_started_before);
        list_addElement(localVarQueryParameters,keyPairQuery_started_before);
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
    char *keyQuery_release = NULL;
    char * valueQuery_release = NULL;
    keyValuePair_t *keyPairQuery_release = 0;
    if (release)
    {
        keyQuery_release = strdup("release");
        valueQuery_release = strdup((release));
        keyPairQuery_release = keyValuePair_create(keyQuery_release, valueQuery_release);
        list_addElement(localVarQueryParameters,keyPairQuery_release);
    }

    // query parameters
    char *keyQuery_min_cost_micros = NULL;
    long valueQuery_min_cost_micros ;
    keyValuePair_t *keyPairQuery_min_cost_micros = 0;
    if (min_cost_micros)
    {
        keyQuery_min_cost_micros = strdup("min_cost_micros");
        valueQuery_min_cost_micros = (min_cost_micros);
        keyPairQuery_min_cost_micros = keyValuePair_create(keyQuery_min_cost_micros, &valueQuery_min_cost_micros);
        list_addElement(localVarQueryParameters,keyPairQuery_min_cost_micros);
    }

    // query parameters
    char *keyQuery_max_cost_micros = NULL;
    long valueQuery_max_cost_micros ;
    keyValuePair_t *keyPairQuery_max_cost_micros = 0;
    if (max_cost_micros)
    {
        keyQuery_max_cost_micros = strdup("max_cost_micros");
        valueQuery_max_cost_micros = (max_cost_micros);
        keyPairQuery_max_cost_micros = keyValuePair_create(keyQuery_max_cost_micros, &valueQuery_max_cost_micros);
        list_addElement(localVarQueryParameters,keyPairQuery_max_cost_micros);
    }

    // query parameters
    char *keyQuery_min_latency_ms = NULL;
    long valueQuery_min_latency_ms ;
    keyValuePair_t *keyPairQuery_min_latency_ms = 0;
    if (min_latency_ms)
    {
        keyQuery_min_latency_ms = strdup("min_latency_ms");
        valueQuery_min_latency_ms = (min_latency_ms);
        keyPairQuery_min_latency_ms = keyValuePair_create(keyQuery_min_latency_ms, &valueQuery_min_latency_ms);
        list_addElement(localVarQueryParameters,keyPairQuery_min_latency_ms);
    }

    // query parameters
    char *keyQuery_max_latency_ms = NULL;
    long valueQuery_max_latency_ms ;
    keyValuePair_t *keyPairQuery_max_latency_ms = 0;
    if (max_latency_ms)
    {
        keyQuery_max_latency_ms = strdup("max_latency_ms");
        valueQuery_max_latency_ms = (max_latency_ms);
        keyPairQuery_max_latency_ms = keyValuePair_create(keyQuery_max_latency_ms, &valueQuery_max_latency_ms);
        list_addElement(localVarQueryParameters,keyPairQuery_max_latency_ms);
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

    // query parameters
    char *keyQuery_cursor = NULL;
    char * valueQuery_cursor = NULL;
    keyValuePair_t *keyPairQuery_cursor = 0;
    if (cursor)
    {
        keyQuery_cursor = strdup("cursor");
        valueQuery_cursor = strdup((cursor));
        keyPairQuery_cursor = keyValuePair_create(keyQuery_cursor, valueQuery_cursor);
        list_addElement(localVarQueryParameters,keyPairQuery_cursor);
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
    //    printf("%s\n","List trace run summaries");
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
    page_run_summary_t *elementToReturn = NULL;
    if(apiClient->response_code >= 200 && apiClient->response_code < 300) {
        cJSON *TracesAPIlocalVarJSON = cJSON_Parse(apiClient->dataReceived);
        elementToReturn = page_run_summary_parseFromJSON(TracesAPIlocalVarJSON);
        cJSON_Delete(TracesAPIlocalVarJSON);
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
    if(keyQuery_started_after){
        free(keyQuery_started_after);
        keyQuery_started_after = NULL;
    }
    if(valueQuery_started_after){
        free(valueQuery_started_after);
        valueQuery_started_after = NULL;
    }
    if(keyPairQuery_started_after){
        keyValuePair_free(keyPairQuery_started_after);
        keyPairQuery_started_after = NULL;
    }
    if(keyQuery_started_before){
        free(keyQuery_started_before);
        keyQuery_started_before = NULL;
    }
    if(valueQuery_started_before){
        free(valueQuery_started_before);
        valueQuery_started_before = NULL;
    }
    if(keyPairQuery_started_before){
        keyValuePair_free(keyPairQuery_started_before);
        keyPairQuery_started_before = NULL;
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
    if(keyQuery_release){
        free(keyQuery_release);
        keyQuery_release = NULL;
    }
    if(valueQuery_release){
        free(valueQuery_release);
        valueQuery_release = NULL;
    }
    if(keyPairQuery_release){
        keyValuePair_free(keyPairQuery_release);
        keyPairQuery_release = NULL;
    }
    if(keyQuery_min_cost_micros){
        free(keyQuery_min_cost_micros);
        keyQuery_min_cost_micros = NULL;
    }
    if(keyPairQuery_min_cost_micros){
        keyValuePair_free(keyPairQuery_min_cost_micros);
        keyPairQuery_min_cost_micros = NULL;
    }
    if(keyQuery_max_cost_micros){
        free(keyQuery_max_cost_micros);
        keyQuery_max_cost_micros = NULL;
    }
    if(keyPairQuery_max_cost_micros){
        keyValuePair_free(keyPairQuery_max_cost_micros);
        keyPairQuery_max_cost_micros = NULL;
    }
    if(keyQuery_min_latency_ms){
        free(keyQuery_min_latency_ms);
        keyQuery_min_latency_ms = NULL;
    }
    if(keyPairQuery_min_latency_ms){
        keyValuePair_free(keyPairQuery_min_latency_ms);
        keyPairQuery_min_latency_ms = NULL;
    }
    if(keyQuery_max_latency_ms){
        free(keyQuery_max_latency_ms);
        keyQuery_max_latency_ms = NULL;
    }
    if(keyPairQuery_max_latency_ms){
        keyValuePair_free(keyPairQuery_max_latency_ms);
        keyPairQuery_max_latency_ms = NULL;
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
    if(keyQuery_cursor){
        free(keyQuery_cursor);
        keyQuery_cursor = NULL;
    }
    if(valueQuery_cursor){
        free(valueQuery_cursor);
        valueQuery_cursor = NULL;
    }
    if(keyPairQuery_cursor){
        keyValuePair_free(keyPairQuery_cursor);
        keyPairQuery_cursor = NULL;
    }
    return elementToReturn;
end:
    free(localVarPath);
    return NULL;

}

