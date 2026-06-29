#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt_variable.h"



static prompt_variable_t *prompt_variable_create_internal(
    char *_default,
    char *description,
    char *name,
    int required
    ) {
    prompt_variable_t *prompt_variable_local_var = malloc(sizeof(prompt_variable_t));
    if (!prompt_variable_local_var) {
        return NULL;
    }
    prompt_variable_local_var->_default = _default;
    prompt_variable_local_var->description = description;
    prompt_variable_local_var->name = name;
    prompt_variable_local_var->required = required;

    prompt_variable_local_var->_library_owned = 1;
    return prompt_variable_local_var;
}

__attribute__((deprecated)) prompt_variable_t *prompt_variable_create(
    char *_default,
    char *description,
    char *name,
    int required
    ) {
    return prompt_variable_create_internal (
        _default,
        description,
        name,
        required
        );
}

void prompt_variable_free(prompt_variable_t *prompt_variable) {
    if(NULL == prompt_variable){
        return ;
    }
    if(prompt_variable->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_variable_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt_variable->_default) {
        free(prompt_variable->_default);
        prompt_variable->_default = NULL;
    }
    if (prompt_variable->description) {
        free(prompt_variable->description);
        prompt_variable->description = NULL;
    }
    if (prompt_variable->name) {
        free(prompt_variable->name);
        prompt_variable->name = NULL;
    }
    free(prompt_variable);
}

cJSON *prompt_variable_convertToJSON(prompt_variable_t *prompt_variable) {
    cJSON *item = cJSON_CreateObject();

    // prompt_variable->_default
    if(prompt_variable->_default) {
    if(cJSON_AddStringToObject(item, "default", prompt_variable->_default) == NULL) {
    goto fail; //String
    }
    }


    // prompt_variable->description
    if(prompt_variable->description) {
    if(cJSON_AddStringToObject(item, "description", prompt_variable->description) == NULL) {
    goto fail; //String
    }
    }


    // prompt_variable->name
    if (!prompt_variable->name) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "name", prompt_variable->name) == NULL) {
    goto fail; //String
    }


    // prompt_variable->required
    if (!prompt_variable->required) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "required", prompt_variable->required) == NULL) {
    goto fail; //Bool
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_variable_t *prompt_variable_parseFromJSON(cJSON *prompt_variableJSON){

    prompt_variable_t *prompt_variable_local_var = NULL;

    // prompt_variable->_default
    cJSON *_default = cJSON_GetObjectItemCaseSensitive(prompt_variableJSON, "default");
    if (cJSON_IsNull(_default)) {
        _default = NULL;
    }
    if (_default) { 
    if(!cJSON_IsString(_default) && !cJSON_IsNull(_default))
    {
    goto end; //String
    }
    }

    // prompt_variable->description
    cJSON *description = cJSON_GetObjectItemCaseSensitive(prompt_variableJSON, "description");
    if (cJSON_IsNull(description)) {
        description = NULL;
    }
    if (description) { 
    if(!cJSON_IsString(description) && !cJSON_IsNull(description))
    {
    goto end; //String
    }
    }

    // prompt_variable->name
    cJSON *name = cJSON_GetObjectItemCaseSensitive(prompt_variableJSON, "name");
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

    // prompt_variable->required
    cJSON *required = cJSON_GetObjectItemCaseSensitive(prompt_variableJSON, "required");
    if (cJSON_IsNull(required)) {
        required = NULL;
    }
    if (!required) {
        goto end;
    }

    
    if(!cJSON_IsBool(required))
    {
    goto end; //Bool
    }


    prompt_variable_local_var = prompt_variable_create_internal (
        _default && !cJSON_IsNull(_default) ? strdup(_default->valuestring) : NULL,
        description && !cJSON_IsNull(description) ? strdup(description->valuestring) : NULL,
        strdup(name->valuestring),
        required->valueint
        );

    return prompt_variable_local_var;
end:
    return NULL;

}
