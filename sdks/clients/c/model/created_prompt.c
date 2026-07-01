#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "created_prompt.h"



static created_prompt_t *created_prompt_create_internal(
    prompt_t *prompt,
    prompt_version_t *version
    ) {
    created_prompt_t *created_prompt_local_var = malloc(sizeof(created_prompt_t));
    if (!created_prompt_local_var) {
        return NULL;
    }
    created_prompt_local_var->prompt = prompt;
    created_prompt_local_var->version = version;

    created_prompt_local_var->_library_owned = 1;
    return created_prompt_local_var;
}

__attribute__((deprecated)) created_prompt_t *created_prompt_create(
    prompt_t *prompt,
    prompt_version_t *version
    ) {
    return created_prompt_create_internal (
        prompt,
        version
        );
}

void created_prompt_free(created_prompt_t *created_prompt) {
    if(NULL == created_prompt){
        return ;
    }
    if(created_prompt->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "created_prompt_free");
        return ;
    }
    listEntry_t *listEntry;
    if (created_prompt->prompt) {
        prompt_free(created_prompt->prompt);
        created_prompt->prompt = NULL;
    }
    if (created_prompt->version) {
        prompt_version_free(created_prompt->version);
        created_prompt->version = NULL;
    }
    free(created_prompt);
}

cJSON *created_prompt_convertToJSON(created_prompt_t *created_prompt) {
    cJSON *item = cJSON_CreateObject();

    // created_prompt->prompt
    if (!created_prompt->prompt) {
        goto fail;
    }
    cJSON *prompt_local_JSON = prompt_convertToJSON(created_prompt->prompt);
    if(prompt_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "prompt", prompt_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // created_prompt->version
    if (!created_prompt->version) {
        goto fail;
    }
    cJSON *version_local_JSON = prompt_version_convertToJSON(created_prompt->version);
    if(version_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "version", version_local_JSON);
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

created_prompt_t *created_prompt_parseFromJSON(cJSON *created_promptJSON){

    created_prompt_t *created_prompt_local_var = NULL;

    // define the local variable for created_prompt->prompt
    prompt_t *prompt_local_nonprim = NULL;

    // define the local variable for created_prompt->version
    prompt_version_t *version_local_nonprim = NULL;

    // created_prompt->prompt
    cJSON *prompt = cJSON_GetObjectItemCaseSensitive(created_promptJSON, "prompt");
    if (cJSON_IsNull(prompt)) {
        prompt = NULL;
    }
    if (!prompt) {
        goto end;
    }

    
    prompt_local_nonprim = prompt_parseFromJSON(prompt); //nonprimitive

    // created_prompt->version
    cJSON *version = cJSON_GetObjectItemCaseSensitive(created_promptJSON, "version");
    if (cJSON_IsNull(version)) {
        version = NULL;
    }
    if (!version) {
        goto end;
    }

    
    version_local_nonprim = prompt_version_parseFromJSON(version); //nonprimitive


    created_prompt_local_var = created_prompt_create_internal (
        prompt_local_nonprim,
        version_local_nonprim
        );

    return created_prompt_local_var;
end:
    if (prompt_local_nonprim) {
        prompt_free(prompt_local_nonprim);
        prompt_local_nonprim = NULL;
    }
    if (version_local_nonprim) {
        prompt_version_free(version_local_nonprim);
        version_local_nonprim = NULL;
    }
    return NULL;

}
