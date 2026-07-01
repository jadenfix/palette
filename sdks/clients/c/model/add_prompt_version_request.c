#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "add_prompt_version_request.h"



static add_prompt_version_request_t *add_prompt_version_request_create_internal(
    char *created_by,
    char *message,
    prompt_template_t *_template
    ) {
    add_prompt_version_request_t *add_prompt_version_request_local_var = malloc(sizeof(add_prompt_version_request_t));
    if (!add_prompt_version_request_local_var) {
        return NULL;
    }
    add_prompt_version_request_local_var->created_by = created_by;
    add_prompt_version_request_local_var->message = message;
    add_prompt_version_request_local_var->_template = _template;

    add_prompt_version_request_local_var->_library_owned = 1;
    return add_prompt_version_request_local_var;
}

__attribute__((deprecated)) add_prompt_version_request_t *add_prompt_version_request_create(
    char *created_by,
    char *message,
    prompt_template_t *_template
    ) {
    return add_prompt_version_request_create_internal (
        created_by,
        message,
        _template
        );
}

void add_prompt_version_request_free(add_prompt_version_request_t *add_prompt_version_request) {
    if(NULL == add_prompt_version_request){
        return ;
    }
    if(add_prompt_version_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "add_prompt_version_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (add_prompt_version_request->created_by) {
        free(add_prompt_version_request->created_by);
        add_prompt_version_request->created_by = NULL;
    }
    if (add_prompt_version_request->message) {
        free(add_prompt_version_request->message);
        add_prompt_version_request->message = NULL;
    }
    if (add_prompt_version_request->_template) {
        prompt_template_free(add_prompt_version_request->_template);
        add_prompt_version_request->_template = NULL;
    }
    free(add_prompt_version_request);
}

cJSON *add_prompt_version_request_convertToJSON(add_prompt_version_request_t *add_prompt_version_request) {
    cJSON *item = cJSON_CreateObject();

    // add_prompt_version_request->created_by
    if(add_prompt_version_request->created_by) {
    if(cJSON_AddStringToObject(item, "created_by", add_prompt_version_request->created_by) == NULL) {
    goto fail; //String
    }
    }


    // add_prompt_version_request->message
    if(add_prompt_version_request->message) {
    if(cJSON_AddStringToObject(item, "message", add_prompt_version_request->message) == NULL) {
    goto fail; //String
    }
    }


    // add_prompt_version_request->_template
    if (!add_prompt_version_request->_template) {
        goto fail;
    }
    cJSON *_template_local_JSON = prompt_template_convertToJSON(add_prompt_version_request->_template);
    if(_template_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "template", _template_local_JSON);
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

add_prompt_version_request_t *add_prompt_version_request_parseFromJSON(cJSON *add_prompt_version_requestJSON){

    add_prompt_version_request_t *add_prompt_version_request_local_var = NULL;

    // define the local variable for add_prompt_version_request->_template
    prompt_template_t *_template_local_nonprim = NULL;

    // add_prompt_version_request->created_by
    cJSON *created_by = cJSON_GetObjectItemCaseSensitive(add_prompt_version_requestJSON, "created_by");
    if (cJSON_IsNull(created_by)) {
        created_by = NULL;
    }
    if (created_by) { 
    if(!cJSON_IsString(created_by) && !cJSON_IsNull(created_by))
    {
    goto end; //String
    }
    }

    // add_prompt_version_request->message
    cJSON *message = cJSON_GetObjectItemCaseSensitive(add_prompt_version_requestJSON, "message");
    if (cJSON_IsNull(message)) {
        message = NULL;
    }
    if (message) { 
    if(!cJSON_IsString(message) && !cJSON_IsNull(message))
    {
    goto end; //String
    }
    }

    // add_prompt_version_request->_template
    cJSON *_template = cJSON_GetObjectItemCaseSensitive(add_prompt_version_requestJSON, "template");
    if (cJSON_IsNull(_template)) {
        _template = NULL;
    }
    if (!_template) {
        goto end;
    }

    
    _template_local_nonprim = prompt_template_parseFromJSON(_template); //nonprimitive


    add_prompt_version_request_local_var = add_prompt_version_request_create_internal (
        created_by && !cJSON_IsNull(created_by) ? strdup(created_by->valuestring) : NULL,
        message && !cJSON_IsNull(message) ? strdup(message->valuestring) : NULL,
        _template_local_nonprim
        );

    return add_prompt_version_request_local_var;
end:
    if (_template_local_nonprim) {
        prompt_template_free(_template_local_nonprim);
        _template_local_nonprim = NULL;
    }
    return NULL;

}
