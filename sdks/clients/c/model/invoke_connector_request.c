#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "invoke_connector_request.h"



static invoke_connector_request_t *invoke_connector_request_create_internal(
    object_t *arguments,
    char *tool
    ) {
    invoke_connector_request_t *invoke_connector_request_local_var = malloc(sizeof(invoke_connector_request_t));
    if (!invoke_connector_request_local_var) {
        return NULL;
    }
    invoke_connector_request_local_var->arguments = arguments;
    invoke_connector_request_local_var->tool = tool;

    invoke_connector_request_local_var->_library_owned = 1;
    return invoke_connector_request_local_var;
}

__attribute__((deprecated)) invoke_connector_request_t *invoke_connector_request_create(
    object_t *arguments,
    char *tool
    ) {
    return invoke_connector_request_create_internal (
        arguments,
        tool
        );
}

void invoke_connector_request_free(invoke_connector_request_t *invoke_connector_request) {
    if(NULL == invoke_connector_request){
        return ;
    }
    if(invoke_connector_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "invoke_connector_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (invoke_connector_request->arguments) {
        object_free(invoke_connector_request->arguments);
        invoke_connector_request->arguments = NULL;
    }
    if (invoke_connector_request->tool) {
        free(invoke_connector_request->tool);
        invoke_connector_request->tool = NULL;
    }
    free(invoke_connector_request);
}

cJSON *invoke_connector_request_convertToJSON(invoke_connector_request_t *invoke_connector_request) {
    cJSON *item = cJSON_CreateObject();

    // invoke_connector_request->arguments
    if(invoke_connector_request->arguments) {
    cJSON *arguments_object = object_convertToJSON(invoke_connector_request->arguments);
    if(arguments_object == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "arguments", arguments_object);
    if(item->child == NULL) {
    goto fail;
    }
    }


    // invoke_connector_request->tool
    if (!invoke_connector_request->tool) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "tool", invoke_connector_request->tool) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

invoke_connector_request_t *invoke_connector_request_parseFromJSON(cJSON *invoke_connector_requestJSON){

    invoke_connector_request_t *invoke_connector_request_local_var = NULL;

    // invoke_connector_request->arguments
    cJSON *arguments = cJSON_GetObjectItemCaseSensitive(invoke_connector_requestJSON, "arguments");
    if (cJSON_IsNull(arguments)) {
        arguments = NULL;
    }
    object_t *arguments_local_object = NULL;
    if (arguments) { 
    arguments_local_object = object_parseFromJSON(arguments); //object
    }

    // invoke_connector_request->tool
    cJSON *tool = cJSON_GetObjectItemCaseSensitive(invoke_connector_requestJSON, "tool");
    if (cJSON_IsNull(tool)) {
        tool = NULL;
    }
    if (!tool) {
        goto end;
    }

    
    if(!cJSON_IsString(tool))
    {
    goto end; //String
    }


    invoke_connector_request_local_var = invoke_connector_request_create_internal (
        arguments ? arguments_local_object : NULL,
        strdup(tool->valuestring)
        );

    return invoke_connector_request_local_var;
end:
    return NULL;

}
