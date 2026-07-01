#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "create_prompt_request.h"



static create_prompt_request_t *create_prompt_request_create_internal(
    char *created_by,
    char *description,
    char *message,
    char *name,
    prompt_template_t *_template
    ) {
    create_prompt_request_t *create_prompt_request_local_var = malloc(sizeof(create_prompt_request_t));
    if (!create_prompt_request_local_var) {
        return NULL;
    }
    create_prompt_request_local_var->created_by = created_by;
    create_prompt_request_local_var->description = description;
    create_prompt_request_local_var->message = message;
    create_prompt_request_local_var->name = name;
    create_prompt_request_local_var->_template = _template;

    create_prompt_request_local_var->_library_owned = 1;
    return create_prompt_request_local_var;
}

__attribute__((deprecated)) create_prompt_request_t *create_prompt_request_create(
    char *created_by,
    char *description,
    char *message,
    char *name,
    prompt_template_t *_template
    ) {
    return create_prompt_request_create_internal (
        created_by,
        description,
        message,
        name,
        _template
        );
}

void create_prompt_request_free(create_prompt_request_t *create_prompt_request) {
    if(NULL == create_prompt_request){
        return ;
    }
    if(create_prompt_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "create_prompt_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (create_prompt_request->created_by) {
        free(create_prompt_request->created_by);
        create_prompt_request->created_by = NULL;
    }
    if (create_prompt_request->description) {
        free(create_prompt_request->description);
        create_prompt_request->description = NULL;
    }
    if (create_prompt_request->message) {
        free(create_prompt_request->message);
        create_prompt_request->message = NULL;
    }
    if (create_prompt_request->name) {
        free(create_prompt_request->name);
        create_prompt_request->name = NULL;
    }
    if (create_prompt_request->_template) {
        prompt_template_free(create_prompt_request->_template);
        create_prompt_request->_template = NULL;
    }
    free(create_prompt_request);
}

cJSON *create_prompt_request_convertToJSON(create_prompt_request_t *create_prompt_request) {
    cJSON *item = cJSON_CreateObject();

    // create_prompt_request->created_by
    if(create_prompt_request->created_by) {
    if(cJSON_AddStringToObject(item, "created_by", create_prompt_request->created_by) == NULL) {
    goto fail; //String
    }
    }


    // create_prompt_request->description
    if(create_prompt_request->description) {
    if(cJSON_AddStringToObject(item, "description", create_prompt_request->description) == NULL) {
    goto fail; //String
    }
    }


    // create_prompt_request->message
    if(create_prompt_request->message) {
    if(cJSON_AddStringToObject(item, "message", create_prompt_request->message) == NULL) {
    goto fail; //String
    }
    }


    // create_prompt_request->name
    if (!create_prompt_request->name) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "name", create_prompt_request->name) == NULL) {
    goto fail; //String
    }


    // create_prompt_request->_template
    if (!create_prompt_request->_template) {
        goto fail;
    }
    cJSON *_template_local_JSON = prompt_template_convertToJSON(create_prompt_request->_template);
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

create_prompt_request_t *create_prompt_request_parseFromJSON(cJSON *create_prompt_requestJSON){

    create_prompt_request_t *create_prompt_request_local_var = NULL;

    // define the local variable for create_prompt_request->_template
    prompt_template_t *_template_local_nonprim = NULL;

    // create_prompt_request->created_by
    cJSON *created_by = cJSON_GetObjectItemCaseSensitive(create_prompt_requestJSON, "created_by");
    if (cJSON_IsNull(created_by)) {
        created_by = NULL;
    }
    if (created_by) { 
    if(!cJSON_IsString(created_by) && !cJSON_IsNull(created_by))
    {
    goto end; //String
    }
    }

    // create_prompt_request->description
    cJSON *description = cJSON_GetObjectItemCaseSensitive(create_prompt_requestJSON, "description");
    if (cJSON_IsNull(description)) {
        description = NULL;
    }
    if (description) { 
    if(!cJSON_IsString(description) && !cJSON_IsNull(description))
    {
    goto end; //String
    }
    }

    // create_prompt_request->message
    cJSON *message = cJSON_GetObjectItemCaseSensitive(create_prompt_requestJSON, "message");
    if (cJSON_IsNull(message)) {
        message = NULL;
    }
    if (message) { 
    if(!cJSON_IsString(message) && !cJSON_IsNull(message))
    {
    goto end; //String
    }
    }

    // create_prompt_request->name
    cJSON *name = cJSON_GetObjectItemCaseSensitive(create_prompt_requestJSON, "name");
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

    // create_prompt_request->_template
    cJSON *_template = cJSON_GetObjectItemCaseSensitive(create_prompt_requestJSON, "template");
    if (cJSON_IsNull(_template)) {
        _template = NULL;
    }
    if (!_template) {
        goto end;
    }

    
    _template_local_nonprim = prompt_template_parseFromJSON(_template); //nonprimitive


    create_prompt_request_local_var = create_prompt_request_create_internal (
        created_by && !cJSON_IsNull(created_by) ? strdup(created_by->valuestring) : NULL,
        description && !cJSON_IsNull(description) ? strdup(description->valuestring) : NULL,
        message && !cJSON_IsNull(message) ? strdup(message->valuestring) : NULL,
        strdup(name->valuestring),
        _template_local_nonprim
        );

    return create_prompt_request_local_var;
end:
    if (_template_local_nonprim) {
        prompt_template_free(_template_local_nonprim);
        _template_local_nonprim = NULL;
    }
    return NULL;

}
