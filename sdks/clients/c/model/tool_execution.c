#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "tool_execution.h"



static tool_execution_t *tool_execution_create_internal(
    object_t *data,
    char *error,
    char *log_id,
    int successful
    ) {
    tool_execution_t *tool_execution_local_var = malloc(sizeof(tool_execution_t));
    if (!tool_execution_local_var) {
        return NULL;
    }
    tool_execution_local_var->data = data;
    tool_execution_local_var->error = error;
    tool_execution_local_var->log_id = log_id;
    tool_execution_local_var->successful = successful;

    tool_execution_local_var->_library_owned = 1;
    return tool_execution_local_var;
}

__attribute__((deprecated)) tool_execution_t *tool_execution_create(
    object_t *data,
    char *error,
    char *log_id,
    int successful
    ) {
    return tool_execution_create_internal (
        data,
        error,
        log_id,
        successful
        );
}

void tool_execution_free(tool_execution_t *tool_execution) {
    if(NULL == tool_execution){
        return ;
    }
    if(tool_execution->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "tool_execution_free");
        return ;
    }
    listEntry_t *listEntry;
    if (tool_execution->data) {
        object_free(tool_execution->data);
        tool_execution->data = NULL;
    }
    if (tool_execution->error) {
        free(tool_execution->error);
        tool_execution->error = NULL;
    }
    if (tool_execution->log_id) {
        free(tool_execution->log_id);
        tool_execution->log_id = NULL;
    }
    free(tool_execution);
}

cJSON *tool_execution_convertToJSON(tool_execution_t *tool_execution) {
    cJSON *item = cJSON_CreateObject();

    // tool_execution->data
    if(tool_execution->data) {
    cJSON *data_object = object_convertToJSON(tool_execution->data);
    if(data_object == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "data", data_object);
    if(item->child == NULL) {
    goto fail;
    }
    }


    // tool_execution->error
    if(tool_execution->error) {
    if(cJSON_AddStringToObject(item, "error", tool_execution->error) == NULL) {
    goto fail; //String
    }
    }


    // tool_execution->log_id
    if(tool_execution->log_id) {
    if(cJSON_AddStringToObject(item, "log_id", tool_execution->log_id) == NULL) {
    goto fail; //String
    }
    }


    // tool_execution->successful
    if (!tool_execution->successful) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "successful", tool_execution->successful) == NULL) {
    goto fail; //Bool
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

tool_execution_t *tool_execution_parseFromJSON(cJSON *tool_executionJSON){

    tool_execution_t *tool_execution_local_var = NULL;

    // tool_execution->data
    cJSON *data = cJSON_GetObjectItemCaseSensitive(tool_executionJSON, "data");
    if (cJSON_IsNull(data)) {
        data = NULL;
    }
    object_t *data_local_object = NULL;
    if (data) { 
    data_local_object = object_parseFromJSON(data); //object
    }

    // tool_execution->error
    cJSON *error = cJSON_GetObjectItemCaseSensitive(tool_executionJSON, "error");
    if (cJSON_IsNull(error)) {
        error = NULL;
    }
    if (error) { 
    if(!cJSON_IsString(error) && !cJSON_IsNull(error))
    {
    goto end; //String
    }
    }

    // tool_execution->log_id
    cJSON *log_id = cJSON_GetObjectItemCaseSensitive(tool_executionJSON, "log_id");
    if (cJSON_IsNull(log_id)) {
        log_id = NULL;
    }
    if (log_id) { 
    if(!cJSON_IsString(log_id) && !cJSON_IsNull(log_id))
    {
    goto end; //String
    }
    }

    // tool_execution->successful
    cJSON *successful = cJSON_GetObjectItemCaseSensitive(tool_executionJSON, "successful");
    if (cJSON_IsNull(successful)) {
        successful = NULL;
    }
    if (!successful) {
        goto end;
    }

    
    if(!cJSON_IsBool(successful))
    {
    goto end; //Bool
    }


    tool_execution_local_var = tool_execution_create_internal (
        data ? data_local_object : NULL,
        error && !cJSON_IsNull(error) ? strdup(error->valuestring) : NULL,
        log_id && !cJSON_IsNull(log_id) ? strdup(log_id->valuestring) : NULL,
        successful->valueint
        );

    return tool_execution_local_var;
end:
    return NULL;

}
