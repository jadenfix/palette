#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "connect_connector_request.h"



static connect_connector_request_t *connect_connector_request_create_internal(
    char *toolkit
    ) {
    connect_connector_request_t *connect_connector_request_local_var = malloc(sizeof(connect_connector_request_t));
    if (!connect_connector_request_local_var) {
        return NULL;
    }
    connect_connector_request_local_var->toolkit = toolkit;

    connect_connector_request_local_var->_library_owned = 1;
    return connect_connector_request_local_var;
}

__attribute__((deprecated)) connect_connector_request_t *connect_connector_request_create(
    char *toolkit
    ) {
    return connect_connector_request_create_internal (
        toolkit
        );
}

void connect_connector_request_free(connect_connector_request_t *connect_connector_request) {
    if(NULL == connect_connector_request){
        return ;
    }
    if(connect_connector_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "connect_connector_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (connect_connector_request->toolkit) {
        free(connect_connector_request->toolkit);
        connect_connector_request->toolkit = NULL;
    }
    free(connect_connector_request);
}

cJSON *connect_connector_request_convertToJSON(connect_connector_request_t *connect_connector_request) {
    cJSON *item = cJSON_CreateObject();

    // connect_connector_request->toolkit
    if (!connect_connector_request->toolkit) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "toolkit", connect_connector_request->toolkit) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

connect_connector_request_t *connect_connector_request_parseFromJSON(cJSON *connect_connector_requestJSON){

    connect_connector_request_t *connect_connector_request_local_var = NULL;

    // connect_connector_request->toolkit
    cJSON *toolkit = cJSON_GetObjectItemCaseSensitive(connect_connector_requestJSON, "toolkit");
    if (cJSON_IsNull(toolkit)) {
        toolkit = NULL;
    }
    if (!toolkit) {
        goto end;
    }

    
    if(!cJSON_IsString(toolkit))
    {
    goto end; //String
    }


    connect_connector_request_local_var = connect_connector_request_create_internal (
        strdup(toolkit->valuestring)
        );

    return connect_connector_request_local_var;
end:
    return NULL;

}
