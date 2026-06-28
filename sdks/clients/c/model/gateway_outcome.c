#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "gateway_outcome.h"



static gateway_outcome_t *gateway_outcome_create_internal(
    int attempts,
    int cached,
    money_t *cost,
    model_ref_t *model,
    char *provider,
    char *request_hash,
    chat_completion_response_t *response,
    token_counts_t *tokens
    ) {
    gateway_outcome_t *gateway_outcome_local_var = malloc(sizeof(gateway_outcome_t));
    if (!gateway_outcome_local_var) {
        return NULL;
    }
    gateway_outcome_local_var->attempts = attempts;
    gateway_outcome_local_var->cached = cached;
    gateway_outcome_local_var->cost = cost;
    gateway_outcome_local_var->model = model;
    gateway_outcome_local_var->provider = provider;
    gateway_outcome_local_var->request_hash = request_hash;
    gateway_outcome_local_var->response = response;
    gateway_outcome_local_var->tokens = tokens;

    gateway_outcome_local_var->_library_owned = 1;
    return gateway_outcome_local_var;
}

__attribute__((deprecated)) gateway_outcome_t *gateway_outcome_create(
    int attempts,
    int cached,
    money_t *cost,
    model_ref_t *model,
    char *provider,
    char *request_hash,
    chat_completion_response_t *response,
    token_counts_t *tokens
    ) {
    return gateway_outcome_create_internal (
        attempts,
        cached,
        cost,
        model,
        provider,
        request_hash,
        response,
        tokens
        );
}

void gateway_outcome_free(gateway_outcome_t *gateway_outcome) {
    if(NULL == gateway_outcome){
        return ;
    }
    if(gateway_outcome->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "gateway_outcome_free");
        return ;
    }
    listEntry_t *listEntry;
    if (gateway_outcome->cost) {
        money_free(gateway_outcome->cost);
        gateway_outcome->cost = NULL;
    }
    if (gateway_outcome->model) {
        model_ref_free(gateway_outcome->model);
        gateway_outcome->model = NULL;
    }
    if (gateway_outcome->provider) {
        free(gateway_outcome->provider);
        gateway_outcome->provider = NULL;
    }
    if (gateway_outcome->request_hash) {
        free(gateway_outcome->request_hash);
        gateway_outcome->request_hash = NULL;
    }
    if (gateway_outcome->response) {
        chat_completion_response_free(gateway_outcome->response);
        gateway_outcome->response = NULL;
    }
    if (gateway_outcome->tokens) {
        token_counts_free(gateway_outcome->tokens);
        gateway_outcome->tokens = NULL;
    }
    free(gateway_outcome);
}

cJSON *gateway_outcome_convertToJSON(gateway_outcome_t *gateway_outcome) {
    cJSON *item = cJSON_CreateObject();

    // gateway_outcome->attempts
    if (!gateway_outcome->attempts) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "attempts", gateway_outcome->attempts) == NULL) {
    goto fail; //Numeric
    }


    // gateway_outcome->cached
    if (!gateway_outcome->cached) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "cached", gateway_outcome->cached) == NULL) {
    goto fail; //Bool
    }


    // gateway_outcome->cost
    if (!gateway_outcome->cost) {
        goto fail;
    }
    cJSON *cost_local_JSON = money_convertToJSON(gateway_outcome->cost);
    if(cost_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "cost", cost_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // gateway_outcome->model
    if (!gateway_outcome->model) {
        goto fail;
    }
    cJSON *model_local_JSON = model_ref_convertToJSON(gateway_outcome->model);
    if(model_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "model", model_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // gateway_outcome->provider
    if (!gateway_outcome->provider) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "provider", gateway_outcome->provider) == NULL) {
    goto fail; //String
    }


    // gateway_outcome->request_hash
    if (!gateway_outcome->request_hash) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "request_hash", gateway_outcome->request_hash) == NULL) {
    goto fail; //String
    }


    // gateway_outcome->response
    if (!gateway_outcome->response) {
        goto fail;
    }
    cJSON *response_local_JSON = chat_completion_response_convertToJSON(gateway_outcome->response);
    if(response_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "response", response_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // gateway_outcome->tokens
    if (!gateway_outcome->tokens) {
        goto fail;
    }
    cJSON *tokens_local_JSON = token_counts_convertToJSON(gateway_outcome->tokens);
    if(tokens_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "tokens", tokens_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

gateway_outcome_t *gateway_outcome_parseFromJSON(cJSON *gateway_outcomeJSON){

    gateway_outcome_t *gateway_outcome_local_var = NULL;

    // define the local variable for gateway_outcome->cost
    money_t *cost_local_nonprim = NULL;

    // define the local variable for gateway_outcome->model
    model_ref_t *model_local_nonprim = NULL;

    // define the local variable for gateway_outcome->response
    chat_completion_response_t *response_local_nonprim = NULL;

    // define the local variable for gateway_outcome->tokens
    token_counts_t *tokens_local_nonprim = NULL;

    // gateway_outcome->attempts
    cJSON *attempts = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "attempts");
    if (cJSON_IsNull(attempts)) {
        attempts = NULL;
    }
    if (!attempts) {
        goto end;
    }

    
    if(!cJSON_IsNumber(attempts))
    {
    goto end; //Numeric
    }

    // gateway_outcome->cached
    cJSON *cached = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "cached");
    if (cJSON_IsNull(cached)) {
        cached = NULL;
    }
    if (!cached) {
        goto end;
    }

    
    if(!cJSON_IsBool(cached))
    {
    goto end; //Bool
    }

    // gateway_outcome->cost
    cJSON *cost = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "cost");
    if (cJSON_IsNull(cost)) {
        cost = NULL;
    }
    if (!cost) {
        goto end;
    }

    
    cost_local_nonprim = money_parseFromJSON(cost); //nonprimitive

    // gateway_outcome->model
    cJSON *model = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "model");
    if (cJSON_IsNull(model)) {
        model = NULL;
    }
    if (!model) {
        goto end;
    }

    
    model_local_nonprim = model_ref_parseFromJSON(model); //nonprimitive

    // gateway_outcome->provider
    cJSON *provider = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "provider");
    if (cJSON_IsNull(provider)) {
        provider = NULL;
    }
    if (!provider) {
        goto end;
    }

    
    if(!cJSON_IsString(provider))
    {
    goto end; //String
    }

    // gateway_outcome->request_hash
    cJSON *request_hash = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "request_hash");
    if (cJSON_IsNull(request_hash)) {
        request_hash = NULL;
    }
    if (!request_hash) {
        goto end;
    }

    
    if(!cJSON_IsString(request_hash))
    {
    goto end; //String
    }

    // gateway_outcome->response
    cJSON *response = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "response");
    if (cJSON_IsNull(response)) {
        response = NULL;
    }
    if (!response) {
        goto end;
    }

    
    response_local_nonprim = chat_completion_response_parseFromJSON(response); //nonprimitive

    // gateway_outcome->tokens
    cJSON *tokens = cJSON_GetObjectItemCaseSensitive(gateway_outcomeJSON, "tokens");
    if (cJSON_IsNull(tokens)) {
        tokens = NULL;
    }
    if (!tokens) {
        goto end;
    }

    
    tokens_local_nonprim = token_counts_parseFromJSON(tokens); //nonprimitive


    gateway_outcome_local_var = gateway_outcome_create_internal (
        attempts->valuedouble,
        cached->valueint,
        cost_local_nonprim,
        model_local_nonprim,
        strdup(provider->valuestring),
        strdup(request_hash->valuestring),
        response_local_nonprim,
        tokens_local_nonprim
        );

    return gateway_outcome_local_var;
end:
    if (cost_local_nonprim) {
        money_free(cost_local_nonprim);
        cost_local_nonprim = NULL;
    }
    if (model_local_nonprim) {
        model_ref_free(model_local_nonprim);
        model_local_nonprim = NULL;
    }
    if (response_local_nonprim) {
        chat_completion_response_free(response_local_nonprim);
        response_local_nonprim = NULL;
    }
    if (tokens_local_nonprim) {
        token_counts_free(tokens_local_nonprim);
        tokens_local_nonprim = NULL;
    }
    return NULL;

}
