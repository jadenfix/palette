#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt_version_metadata.h"



static prompt_version_metadata_t *prompt_version_metadata_create_internal(
    char *created_at,
    char *created_by,
    char *message
    ) {
    prompt_version_metadata_t *prompt_version_metadata_local_var = malloc(sizeof(prompt_version_metadata_t));
    if (!prompt_version_metadata_local_var) {
        return NULL;
    }
    prompt_version_metadata_local_var->created_at = created_at;
    prompt_version_metadata_local_var->created_by = created_by;
    prompt_version_metadata_local_var->message = message;

    prompt_version_metadata_local_var->_library_owned = 1;
    return prompt_version_metadata_local_var;
}

__attribute__((deprecated)) prompt_version_metadata_t *prompt_version_metadata_create(
    char *created_at,
    char *created_by,
    char *message
    ) {
    return prompt_version_metadata_create_internal (
        created_at,
        created_by,
        message
        );
}

void prompt_version_metadata_free(prompt_version_metadata_t *prompt_version_metadata) {
    if(NULL == prompt_version_metadata){
        return ;
    }
    if(prompt_version_metadata->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_version_metadata_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt_version_metadata->created_at) {
        free(prompt_version_metadata->created_at);
        prompt_version_metadata->created_at = NULL;
    }
    if (prompt_version_metadata->created_by) {
        free(prompt_version_metadata->created_by);
        prompt_version_metadata->created_by = NULL;
    }
    if (prompt_version_metadata->message) {
        free(prompt_version_metadata->message);
        prompt_version_metadata->message = NULL;
    }
    free(prompt_version_metadata);
}

cJSON *prompt_version_metadata_convertToJSON(prompt_version_metadata_t *prompt_version_metadata) {
    cJSON *item = cJSON_CreateObject();

    // prompt_version_metadata->created_at
    if (!prompt_version_metadata->created_at) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "created_at", prompt_version_metadata->created_at) == NULL) {
    goto fail; //Date-Time
    }


    // prompt_version_metadata->created_by
    if(prompt_version_metadata->created_by) {
    if(cJSON_AddStringToObject(item, "created_by", prompt_version_metadata->created_by) == NULL) {
    goto fail; //String
    }
    }


    // prompt_version_metadata->message
    if(prompt_version_metadata->message) {
    if(cJSON_AddStringToObject(item, "message", prompt_version_metadata->message) == NULL) {
    goto fail; //String
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_version_metadata_t *prompt_version_metadata_parseFromJSON(cJSON *prompt_version_metadataJSON){

    prompt_version_metadata_t *prompt_version_metadata_local_var = NULL;

    // prompt_version_metadata->created_at
    cJSON *created_at = cJSON_GetObjectItemCaseSensitive(prompt_version_metadataJSON, "created_at");
    if (cJSON_IsNull(created_at)) {
        created_at = NULL;
    }
    if (!created_at) {
        goto end;
    }

    
    if(!cJSON_IsString(created_at) && !cJSON_IsNull(created_at))
    {
    goto end; //DateTime
    }

    // prompt_version_metadata->created_by
    cJSON *created_by = cJSON_GetObjectItemCaseSensitive(prompt_version_metadataJSON, "created_by");
    if (cJSON_IsNull(created_by)) {
        created_by = NULL;
    }
    if (created_by) { 
    if(!cJSON_IsString(created_by) && !cJSON_IsNull(created_by))
    {
    goto end; //String
    }
    }

    // prompt_version_metadata->message
    cJSON *message = cJSON_GetObjectItemCaseSensitive(prompt_version_metadataJSON, "message");
    if (cJSON_IsNull(message)) {
        message = NULL;
    }
    if (message) { 
    if(!cJSON_IsString(message) && !cJSON_IsNull(message))
    {
    goto end; //String
    }
    }


    prompt_version_metadata_local_var = prompt_version_metadata_create_internal (
        strdup(created_at->valuestring),
        created_by && !cJSON_IsNull(created_by) ? strdup(created_by->valuestring) : NULL,
        message && !cJSON_IsNull(message) ? strdup(message->valuestring) : NULL
        );

    return prompt_version_metadata_local_var;
end:
    return NULL;

}
