#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "perturbation_knobs.h"



static perturbation_knobs_t *perturbation_knobs_create_internal(
    int auth_failure,
    int contradictory_source,
    int prompt_injection,
    int stale_source,
    int timeout,
    int tool_schema_mismatch
    ) {
    perturbation_knobs_t *perturbation_knobs_local_var = malloc(sizeof(perturbation_knobs_t));
    if (!perturbation_knobs_local_var) {
        return NULL;
    }
    perturbation_knobs_local_var->auth_failure = auth_failure;
    perturbation_knobs_local_var->contradictory_source = contradictory_source;
    perturbation_knobs_local_var->prompt_injection = prompt_injection;
    perturbation_knobs_local_var->stale_source = stale_source;
    perturbation_knobs_local_var->timeout = timeout;
    perturbation_knobs_local_var->tool_schema_mismatch = tool_schema_mismatch;

    perturbation_knobs_local_var->_library_owned = 1;
    return perturbation_knobs_local_var;
}

__attribute__((deprecated)) perturbation_knobs_t *perturbation_knobs_create(
    int auth_failure,
    int contradictory_source,
    int prompt_injection,
    int stale_source,
    int timeout,
    int tool_schema_mismatch
    ) {
    return perturbation_knobs_create_internal (
        auth_failure,
        contradictory_source,
        prompt_injection,
        stale_source,
        timeout,
        tool_schema_mismatch
        );
}

void perturbation_knobs_free(perturbation_knobs_t *perturbation_knobs) {
    if(NULL == perturbation_knobs){
        return ;
    }
    if(perturbation_knobs->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "perturbation_knobs_free");
        return ;
    }
    listEntry_t *listEntry;
    free(perturbation_knobs);
}

cJSON *perturbation_knobs_convertToJSON(perturbation_knobs_t *perturbation_knobs) {
    cJSON *item = cJSON_CreateObject();

    // perturbation_knobs->auth_failure
    if (!perturbation_knobs->auth_failure) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "auth_failure", perturbation_knobs->auth_failure) == NULL) {
    goto fail; //Bool
    }


    // perturbation_knobs->contradictory_source
    if (!perturbation_knobs->contradictory_source) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "contradictory_source", perturbation_knobs->contradictory_source) == NULL) {
    goto fail; //Bool
    }


    // perturbation_knobs->prompt_injection
    if (!perturbation_knobs->prompt_injection) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "prompt_injection", perturbation_knobs->prompt_injection) == NULL) {
    goto fail; //Bool
    }


    // perturbation_knobs->stale_source
    if (!perturbation_knobs->stale_source) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "stale_source", perturbation_knobs->stale_source) == NULL) {
    goto fail; //Bool
    }


    // perturbation_knobs->timeout
    if (!perturbation_knobs->timeout) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "timeout", perturbation_knobs->timeout) == NULL) {
    goto fail; //Bool
    }


    // perturbation_knobs->tool_schema_mismatch
    if (!perturbation_knobs->tool_schema_mismatch) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "tool_schema_mismatch", perturbation_knobs->tool_schema_mismatch) == NULL) {
    goto fail; //Bool
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

perturbation_knobs_t *perturbation_knobs_parseFromJSON(cJSON *perturbation_knobsJSON){

    perturbation_knobs_t *perturbation_knobs_local_var = NULL;

    // perturbation_knobs->auth_failure
    cJSON *auth_failure = cJSON_GetObjectItemCaseSensitive(perturbation_knobsJSON, "auth_failure");
    if (cJSON_IsNull(auth_failure)) {
        auth_failure = NULL;
    }
    if (!auth_failure) {
        goto end;
    }

    
    if(!cJSON_IsBool(auth_failure))
    {
    goto end; //Bool
    }

    // perturbation_knobs->contradictory_source
    cJSON *contradictory_source = cJSON_GetObjectItemCaseSensitive(perturbation_knobsJSON, "contradictory_source");
    if (cJSON_IsNull(contradictory_source)) {
        contradictory_source = NULL;
    }
    if (!contradictory_source) {
        goto end;
    }

    
    if(!cJSON_IsBool(contradictory_source))
    {
    goto end; //Bool
    }

    // perturbation_knobs->prompt_injection
    cJSON *prompt_injection = cJSON_GetObjectItemCaseSensitive(perturbation_knobsJSON, "prompt_injection");
    if (cJSON_IsNull(prompt_injection)) {
        prompt_injection = NULL;
    }
    if (!prompt_injection) {
        goto end;
    }

    
    if(!cJSON_IsBool(prompt_injection))
    {
    goto end; //Bool
    }

    // perturbation_knobs->stale_source
    cJSON *stale_source = cJSON_GetObjectItemCaseSensitive(perturbation_knobsJSON, "stale_source");
    if (cJSON_IsNull(stale_source)) {
        stale_source = NULL;
    }
    if (!stale_source) {
        goto end;
    }

    
    if(!cJSON_IsBool(stale_source))
    {
    goto end; //Bool
    }

    // perturbation_knobs->timeout
    cJSON *timeout = cJSON_GetObjectItemCaseSensitive(perturbation_knobsJSON, "timeout");
    if (cJSON_IsNull(timeout)) {
        timeout = NULL;
    }
    if (!timeout) {
        goto end;
    }

    
    if(!cJSON_IsBool(timeout))
    {
    goto end; //Bool
    }

    // perturbation_knobs->tool_schema_mismatch
    cJSON *tool_schema_mismatch = cJSON_GetObjectItemCaseSensitive(perturbation_knobsJSON, "tool_schema_mismatch");
    if (cJSON_IsNull(tool_schema_mismatch)) {
        tool_schema_mismatch = NULL;
    }
    if (!tool_schema_mismatch) {
        goto end;
    }

    
    if(!cJSON_IsBool(tool_schema_mismatch))
    {
    goto end; //Bool
    }


    perturbation_knobs_local_var = perturbation_knobs_create_internal (
        auth_failure->valueint,
        contradictory_source->valueint,
        prompt_injection->valueint,
        stale_source->valueint,
        timeout->valueint,
        tool_schema_mismatch->valueint
        );

    return perturbation_knobs_local_var;
end:
    return NULL;

}
