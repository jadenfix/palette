#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt_version.h"



static prompt_version_t *prompt_version_create_internal(
    prompt_version_metadata_t *metadata,
    char *project_id,
    char *prompt_id,
    prompt_template_t *_template,
    char *tenant_id,
    char *version_id,
    int version_number
    ) {
    prompt_version_t *prompt_version_local_var = malloc(sizeof(prompt_version_t));
    if (!prompt_version_local_var) {
        return NULL;
    }
    prompt_version_local_var->metadata = metadata;
    prompt_version_local_var->project_id = project_id;
    prompt_version_local_var->prompt_id = prompt_id;
    prompt_version_local_var->_template = _template;
    prompt_version_local_var->tenant_id = tenant_id;
    prompt_version_local_var->version_id = version_id;
    prompt_version_local_var->version_number = version_number;

    prompt_version_local_var->_library_owned = 1;
    return prompt_version_local_var;
}

__attribute__((deprecated)) prompt_version_t *prompt_version_create(
    prompt_version_metadata_t *metadata,
    char *project_id,
    char *prompt_id,
    prompt_template_t *_template,
    char *tenant_id,
    char *version_id,
    int version_number
    ) {
    return prompt_version_create_internal (
        metadata,
        project_id,
        prompt_id,
        _template,
        tenant_id,
        version_id,
        version_number
        );
}

void prompt_version_free(prompt_version_t *prompt_version) {
    if(NULL == prompt_version){
        return ;
    }
    if(prompt_version->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_version_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt_version->metadata) {
        prompt_version_metadata_free(prompt_version->metadata);
        prompt_version->metadata = NULL;
    }
    if (prompt_version->project_id) {
        free(prompt_version->project_id);
        prompt_version->project_id = NULL;
    }
    if (prompt_version->prompt_id) {
        free(prompt_version->prompt_id);
        prompt_version->prompt_id = NULL;
    }
    if (prompt_version->_template) {
        prompt_template_free(prompt_version->_template);
        prompt_version->_template = NULL;
    }
    if (prompt_version->tenant_id) {
        free(prompt_version->tenant_id);
        prompt_version->tenant_id = NULL;
    }
    if (prompt_version->version_id) {
        free(prompt_version->version_id);
        prompt_version->version_id = NULL;
    }
    free(prompt_version);
}

cJSON *prompt_version_convertToJSON(prompt_version_t *prompt_version) {
    cJSON *item = cJSON_CreateObject();

    // prompt_version->metadata
    if (!prompt_version->metadata) {
        goto fail;
    }
    cJSON *metadata_local_JSON = prompt_version_metadata_convertToJSON(prompt_version->metadata);
    if(metadata_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "metadata", metadata_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // prompt_version->project_id
    if (!prompt_version->project_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "project_id", prompt_version->project_id) == NULL) {
    goto fail; //String
    }


    // prompt_version->prompt_id
    if (!prompt_version->prompt_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "prompt_id", prompt_version->prompt_id) == NULL) {
    goto fail; //String
    }


    // prompt_version->_template
    if (!prompt_version->_template) {
        goto fail;
    }
    cJSON *_template_local_JSON = prompt_template_convertToJSON(prompt_version->_template);
    if(_template_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "template", _template_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // prompt_version->tenant_id
    if (!prompt_version->tenant_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "tenant_id", prompt_version->tenant_id) == NULL) {
    goto fail; //String
    }


    // prompt_version->version_id
    if (!prompt_version->version_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "version_id", prompt_version->version_id) == NULL) {
    goto fail; //String
    }


    // prompt_version->version_number
    if (!prompt_version->version_number) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "version_number", prompt_version->version_number) == NULL) {
    goto fail; //Numeric
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_version_t *prompt_version_parseFromJSON(cJSON *prompt_versionJSON){

    prompt_version_t *prompt_version_local_var = NULL;

    // define the local variable for prompt_version->metadata
    prompt_version_metadata_t *metadata_local_nonprim = NULL;

    // define the local variable for prompt_version->_template
    prompt_template_t *_template_local_nonprim = NULL;

    // prompt_version->metadata
    cJSON *metadata = cJSON_GetObjectItemCaseSensitive(prompt_versionJSON, "metadata");
    if (cJSON_IsNull(metadata)) {
        metadata = NULL;
    }
    if (!metadata) {
        goto end;
    }

    
    metadata_local_nonprim = prompt_version_metadata_parseFromJSON(metadata); //nonprimitive

    // prompt_version->project_id
    cJSON *project_id = cJSON_GetObjectItemCaseSensitive(prompt_versionJSON, "project_id");
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

    // prompt_version->prompt_id
    cJSON *prompt_id = cJSON_GetObjectItemCaseSensitive(prompt_versionJSON, "prompt_id");
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

    // prompt_version->_template
    cJSON *_template = cJSON_GetObjectItemCaseSensitive(prompt_versionJSON, "template");
    if (cJSON_IsNull(_template)) {
        _template = NULL;
    }
    if (!_template) {
        goto end;
    }

    
    _template_local_nonprim = prompt_template_parseFromJSON(_template); //nonprimitive

    // prompt_version->tenant_id
    cJSON *tenant_id = cJSON_GetObjectItemCaseSensitive(prompt_versionJSON, "tenant_id");
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

    // prompt_version->version_id
    cJSON *version_id = cJSON_GetObjectItemCaseSensitive(prompt_versionJSON, "version_id");
    if (cJSON_IsNull(version_id)) {
        version_id = NULL;
    }
    if (!version_id) {
        goto end;
    }

    
    if(!cJSON_IsString(version_id))
    {
    goto end; //String
    }

    // prompt_version->version_number
    cJSON *version_number = cJSON_GetObjectItemCaseSensitive(prompt_versionJSON, "version_number");
    if (cJSON_IsNull(version_number)) {
        version_number = NULL;
    }
    if (!version_number) {
        goto end;
    }

    
    if(!cJSON_IsNumber(version_number))
    {
    goto end; //Numeric
    }


    prompt_version_local_var = prompt_version_create_internal (
        metadata_local_nonprim,
        strdup(project_id->valuestring),
        strdup(prompt_id->valuestring),
        _template_local_nonprim,
        strdup(tenant_id->valuestring),
        strdup(version_id->valuestring),
        version_number->valuedouble
        );

    return prompt_version_local_var;
end:
    if (metadata_local_nonprim) {
        prompt_version_metadata_free(metadata_local_nonprim);
        metadata_local_nonprim = NULL;
    }
    if (_template_local_nonprim) {
        prompt_template_free(_template_local_nonprim);
        _template_local_nonprim = NULL;
    }
    return NULL;

}
