#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "import_source_http_request.h"



static import_source_http_request_t *import_source_http_request_create_internal(
    any_type_t *payload,
    char *source
    ) {
    import_source_http_request_t *import_source_http_request_local_var = malloc(sizeof(import_source_http_request_t));
    if (!import_source_http_request_local_var) {
        return NULL;
    }
    import_source_http_request_local_var->payload = payload;
    import_source_http_request_local_var->source = source;

    import_source_http_request_local_var->_library_owned = 1;
    return import_source_http_request_local_var;
}

__attribute__((deprecated)) import_source_http_request_t *import_source_http_request_create(
    any_type_t *payload,
    char *source
    ) {
    return import_source_http_request_create_internal (
        payload,
        source
        );
}

void import_source_http_request_free(import_source_http_request_t *import_source_http_request) {
    if(NULL == import_source_http_request){
        return ;
    }
    if(import_source_http_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "import_source_http_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (import_source_http_request->payload) {
        _free(import_source_http_request->payload);
        import_source_http_request->payload = NULL;
    }
    if (import_source_http_request->source) {
        free(import_source_http_request->source);
        import_source_http_request->source = NULL;
    }
    free(import_source_http_request);
}

cJSON *import_source_http_request_convertToJSON(import_source_http_request_t *import_source_http_request) {
    cJSON *item = cJSON_CreateObject();

    // import_source_http_request->payload
    if(import_source_http_request->payload) {
    cJSON *payload_local_JSON = _convertToJSON(import_source_http_request->payload);
    if(payload_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "payload", payload_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }
    }


    // import_source_http_request->source
    if (!import_source_http_request->source) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "source", import_source_http_request->source) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

import_source_http_request_t *import_source_http_request_parseFromJSON(cJSON *import_source_http_requestJSON){

    import_source_http_request_t *import_source_http_request_local_var = NULL;

    // define the local variable for import_source_http_request->payload
    _t *payload_local_nonprim = NULL;

    // import_source_http_request->payload
    cJSON *payload = cJSON_GetObjectItemCaseSensitive(import_source_http_requestJSON, "payload");
    if (cJSON_IsNull(payload)) {
        payload = NULL;
    }
    if (payload) { 
    payload_local_nonprim = _parseFromJSON(payload); //custom
    }

    // import_source_http_request->source
    cJSON *source = cJSON_GetObjectItemCaseSensitive(import_source_http_requestJSON, "source");
    if (cJSON_IsNull(source)) {
        source = NULL;
    }
    if (!source) {
        goto end;
    }

    
    if(!cJSON_IsString(source))
    {
    goto end; //String
    }


    import_source_http_request_local_var = import_source_http_request_create_internal (
        payload ? payload_local_nonprim : NULL,
        strdup(source->valuestring)
        );

    return import_source_http_request_local_var;
end:
    if (payload_local_nonprim) {
        _free(payload_local_nonprim);
        payload_local_nonprim = NULL;
    }
    return NULL;

}
