#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt.h"



static prompt_t *prompt_create_internal(
    char *created_at,
    char *description,
    char *name,
    char *project_id,
    char *prompt_id,
    char *tenant_id,
    char *updated_at
    ) {
    prompt_t *prompt_local_var = malloc(sizeof(prompt_t));
    if (!prompt_local_var) {
        return NULL;
    }
    prompt_local_var->created_at = created_at;
    prompt_local_var->description = description;
    prompt_local_var->name = name;
    prompt_local_var->project_id = project_id;
    prompt_local_var->prompt_id = prompt_id;
    prompt_local_var->tenant_id = tenant_id;
    prompt_local_var->updated_at = updated_at;

    prompt_local_var->_library_owned = 1;
    return prompt_local_var;
}

__attribute__((deprecated)) prompt_t *prompt_create(
    char *created_at,
    char *description,
    char *name,
    char *project_id,
    char *prompt_id,
    char *tenant_id,
    char *updated_at
    ) {
    return prompt_create_internal (
        created_at,
        description,
        name,
        project_id,
        prompt_id,
        tenant_id,
        updated_at
        );
}

void prompt_free(prompt_t *prompt) {
    if(NULL == prompt){
        return ;
    }
    if(prompt->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt->created_at) {
        free(prompt->created_at);
        prompt->created_at = NULL;
    }
    if (prompt->description) {
        free(prompt->description);
        prompt->description = NULL;
    }
    if (prompt->name) {
        free(prompt->name);
        prompt->name = NULL;
    }
    if (prompt->project_id) {
        free(prompt->project_id);
        prompt->project_id = NULL;
    }
    if (prompt->prompt_id) {
        free(prompt->prompt_id);
        prompt->prompt_id = NULL;
    }
    if (prompt->tenant_id) {
        free(prompt->tenant_id);
        prompt->tenant_id = NULL;
    }
    if (prompt->updated_at) {
        free(prompt->updated_at);
        prompt->updated_at = NULL;
    }
    free(prompt);
}

cJSON *prompt_convertToJSON(prompt_t *prompt) {
    cJSON *item = cJSON_CreateObject();

    // prompt->created_at
    if (!prompt->created_at) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "created_at", prompt->created_at) == NULL) {
    goto fail; //Date-Time
    }


    // prompt->description
    if(prompt->description) {
    if(cJSON_AddStringToObject(item, "description", prompt->description) == NULL) {
    goto fail; //String
    }
    }


    // prompt->name
    if (!prompt->name) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "name", prompt->name) == NULL) {
    goto fail; //String
    }


    // prompt->project_id
    if (!prompt->project_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "project_id", prompt->project_id) == NULL) {
    goto fail; //String
    }


    // prompt->prompt_id
    if (!prompt->prompt_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "prompt_id", prompt->prompt_id) == NULL) {
    goto fail; //String
    }


    // prompt->tenant_id
    if (!prompt->tenant_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "tenant_id", prompt->tenant_id) == NULL) {
    goto fail; //String
    }


    // prompt->updated_at
    if (!prompt->updated_at) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "updated_at", prompt->updated_at) == NULL) {
    goto fail; //Date-Time
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_t *prompt_parseFromJSON(cJSON *promptJSON){

    prompt_t *prompt_local_var = NULL;

    // prompt->created_at
    cJSON *created_at = cJSON_GetObjectItemCaseSensitive(promptJSON, "created_at");
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

    // prompt->description
    cJSON *description = cJSON_GetObjectItemCaseSensitive(promptJSON, "description");
    if (cJSON_IsNull(description)) {
        description = NULL;
    }
    if (description) { 
    if(!cJSON_IsString(description) && !cJSON_IsNull(description))
    {
    goto end; //String
    }
    }

    // prompt->name
    cJSON *name = cJSON_GetObjectItemCaseSensitive(promptJSON, "name");
    if (cJSON_IsNull(name)) {
        name = NULL;
    }
    if (!name) {
        goto end;
    }

    
    if(!cJSON_IsString(name))
    {
    goto end; //String
    }

    // prompt->project_id
    cJSON *project_id = cJSON_GetObjectItemCaseSensitive(promptJSON, "project_id");
    if (cJSON_IsNull(project_id)) {
        project_id = NULL;
    }
    if (!project_id) {
        goto end;
    }

    
    if(!cJSON_IsString(project_id))
    {
    goto end; //String
    }

    // prompt->prompt_id
    cJSON *prompt_id = cJSON_GetObjectItemCaseSensitive(promptJSON, "prompt_id");
    if (cJSON_IsNull(prompt_id)) {
        prompt_id = NULL;
    }
    if (!prompt_id) {
        goto end;
    }

    
    if(!cJSON_IsString(prompt_id))
    {
    goto end; //String
    }

    // prompt->tenant_id
    cJSON *tenant_id = cJSON_GetObjectItemCaseSensitive(promptJSON, "tenant_id");
    if (cJSON_IsNull(tenant_id)) {
        tenant_id = NULL;
    }
    if (!tenant_id) {
        goto end;
    }

    
    if(!cJSON_IsString(tenant_id))
    {
    goto end; //String
    }

    // prompt->updated_at
    cJSON *updated_at = cJSON_GetObjectItemCaseSensitive(promptJSON, "updated_at");
    if (cJSON_IsNull(updated_at)) {
        updated_at = NULL;
    }
    if (!updated_at) {
        goto end;
    }

    
    if(!cJSON_IsString(updated_at) && !cJSON_IsNull(updated_at))
    {
    goto end; //DateTime
    }


    prompt_local_var = prompt_create_internal (
        strdup(created_at->valuestring),
        description && !cJSON_IsNull(description) ? strdup(description->valuestring) : NULL,
        strdup(name->valuestring),
        strdup(project_id->valuestring),
        strdup(prompt_id->valuestring),
        strdup(tenant_id->valuestring),
        strdup(updated_at->valuestring)
        );

    return prompt_local_var;
end:
    return NULL;

}
