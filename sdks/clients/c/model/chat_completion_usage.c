#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "chat_completion_usage.h"



static chat_completion_usage_t *chat_completion_usage_create_internal(
    long completion_tokens,
    long prompt_tokens,
    long total_tokens
    ) {
    chat_completion_usage_t *chat_completion_usage_local_var = malloc(sizeof(chat_completion_usage_t));
    if (!chat_completion_usage_local_var) {
        return NULL;
    }
    chat_completion_usage_local_var->completion_tokens = completion_tokens;
    chat_completion_usage_local_var->prompt_tokens = prompt_tokens;
    chat_completion_usage_local_var->total_tokens = total_tokens;

    chat_completion_usage_local_var->_library_owned = 1;
    return chat_completion_usage_local_var;
}

__attribute__((deprecated)) chat_completion_usage_t *chat_completion_usage_create(
    long completion_tokens,
    long prompt_tokens,
    long total_tokens
    ) {
    return chat_completion_usage_create_internal (
        completion_tokens,
        prompt_tokens,
        total_tokens
        );
}

void chat_completion_usage_free(chat_completion_usage_t *chat_completion_usage) {
    if(NULL == chat_completion_usage){
        return ;
    }
    if(chat_completion_usage->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "chat_completion_usage_free");
        return ;
    }
    listEntry_t *listEntry;
    free(chat_completion_usage);
}

cJSON *chat_completion_usage_convertToJSON(chat_completion_usage_t *chat_completion_usage) {
    cJSON *item = cJSON_CreateObject();

    // chat_completion_usage->completion_tokens
    if (!chat_completion_usage->completion_tokens) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "completion_tokens", chat_completion_usage->completion_tokens) == NULL) {
    goto fail; //Numeric
    }


    // chat_completion_usage->prompt_tokens
    if (!chat_completion_usage->prompt_tokens) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "prompt_tokens", chat_completion_usage->prompt_tokens) == NULL) {
    goto fail; //Numeric
    }


    // chat_completion_usage->total_tokens
    if (!chat_completion_usage->total_tokens) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "total_tokens", chat_completion_usage->total_tokens) == NULL) {
    goto fail; //Numeric
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

chat_completion_usage_t *chat_completion_usage_parseFromJSON(cJSON *chat_completion_usageJSON){

    chat_completion_usage_t *chat_completion_usage_local_var = NULL;

    // chat_completion_usage->completion_tokens
    cJSON *completion_tokens = cJSON_GetObjectItemCaseSensitive(chat_completion_usageJSON, "completion_tokens");
    if (cJSON_IsNull(completion_tokens)) {
        completion_tokens = NULL;
    }
    if (!completion_tokens) {
        goto end;
    }

    
    if(!cJSON_IsNumber(completion_tokens))
    {
    goto end; //Numeric
    }

    // chat_completion_usage->prompt_tokens
    cJSON *prompt_tokens = cJSON_GetObjectItemCaseSensitive(chat_completion_usageJSON, "prompt_tokens");
    if (cJSON_IsNull(prompt_tokens)) {
        prompt_tokens = NULL;
    }
    if (!prompt_tokens) {
        goto end;
    }

    
    if(!cJSON_IsNumber(prompt_tokens))
    {
    goto end; //Numeric
    }

    // chat_completion_usage->total_tokens
    cJSON *total_tokens = cJSON_GetObjectItemCaseSensitive(chat_completion_usageJSON, "total_tokens");
    if (cJSON_IsNull(total_tokens)) {
        total_tokens = NULL;
    }
    if (!total_tokens) {
        goto end;
    }

    
    if(!cJSON_IsNumber(total_tokens))
    {
    goto end; //Numeric
    }


    chat_completion_usage_local_var = chat_completion_usage_create_internal (
        completion_tokens->valuedouble,
        prompt_tokens->valuedouble,
        total_tokens->valuedouble
        );

    return chat_completion_usage_local_var;
end:
    return NULL;

}
