#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "create_api_key_http_request.h"


char* create_api_key_http_request_scopes_ToString(beater_api_create_api_key_http_request__e scopes) {
    char *scopesArray[] =  { "NULL", "trace:write", "trace:read", "dataset:write", "scenario:write", "scenario:read", "eval:run", "pii:unmask", "admin" };
    return scopesArray[scopes - 1];
}

beater_api_create_api_key_http_request__e create_api_key_http_request_scopes_FromString(char* scopes) {
    int stringToReturn = 0;
    char *scopesArray[] =  { "NULL", "trace:write", "trace:read", "dataset:write", "scenario:write", "scenario:read", "eval:run", "pii:unmask", "admin" };
    size_t sizeofArray = sizeof(scopesArray) / sizeof(scopesArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(scopes, scopesArray[stringToReturn]) == 0) {
            return stringToReturn + 1;
        }
        stringToReturn++;
    }
    return 0;
}

static create_api_key_http_request_t *create_api_key_http_request_create_internal(
    list_t *scopes
    ) {
    create_api_key_http_request_t *create_api_key_http_request_local_var = malloc(sizeof(create_api_key_http_request_t));
    if (!create_api_key_http_request_local_var) {
        return NULL;
    }
    create_api_key_http_request_local_var->scopes = scopes;

    create_api_key_http_request_local_var->_library_owned = 1;
    return create_api_key_http_request_local_var;
}

__attribute__((deprecated)) create_api_key_http_request_t *create_api_key_http_request_create(
    list_t *scopes
    ) {
    return create_api_key_http_request_create_internal (
        scopes
        );
}

void create_api_key_http_request_free(create_api_key_http_request_t *create_api_key_http_request) {
    if(NULL == create_api_key_http_request){
        return ;
    }
    if(create_api_key_http_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "create_api_key_http_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (create_api_key_http_request->scopes) {
        list_ForEach(listEntry, create_api_key_http_request->scopes) {
            api_scope_free(listEntry->data);
        }
        list_freeList(create_api_key_http_request->scopes);
        create_api_key_http_request->scopes = NULL;
    }
    free(create_api_key_http_request);
}

cJSON *create_api_key_http_request_convertToJSON(create_api_key_http_request_t *create_api_key_http_request) {
    cJSON *item = cJSON_CreateObject();

    // create_api_key_http_request->scopes
    if (beater_api_list_SCOPES_NULL == create_api_key_http_request->scopes) {
        goto fail;
    }
    cJSON *scopes = cJSON_AddArrayToObject(item, "scopes");
    if(scopes == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *scopesListEntry;
    if (create_api_key_http_request->scopes) {
    list_ForEach(scopesListEntry, create_api_key_http_request->scopes) {
    cJSON *itemLocal = api_scope_convertToJSON((beater_api_create_api_key_http_request__e)scopesListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(scopes, itemLocal);
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

create_api_key_http_request_t *create_api_key_http_request_parseFromJSON(cJSON *create_api_key_http_requestJSON){

    create_api_key_http_request_t *create_api_key_http_request_local_var = NULL;

    // define the local list for create_api_key_http_request->scopes
    list_t *scopesList = NULL;

    // create_api_key_http_request->scopes
    cJSON *scopes = cJSON_GetObjectItemCaseSensitive(create_api_key_http_requestJSON, "scopes");
    if (cJSON_IsNull(scopes)) {
        scopes = NULL;
    }
    if (!scopes) {
        goto end;
    }

    
    cJSON *scopes_local_nonprimitive = NULL;
    if(!cJSON_IsArray(scopes)){
        goto end; //nonprimitive container
    }

    scopesList = list_createList();

    cJSON_ArrayForEach(scopes_local_nonprimitive,scopes )
    {
        if(!cJSON_IsObject(scopes_local_nonprimitive)){
            goto end;
        }
        create_api_key_http_request_api_scope_e scopesItem = api_scope_parseFromJSON(scopes_local_nonprimitive);

        list_addElement(scopesList, (void *)scopesItem);
    }


    create_api_key_http_request_local_var = create_api_key_http_request_create_internal (
        scopesList
        );

    return create_api_key_http_request_local_var;
end:
    if (scopesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, scopesList) {
            api_scope_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(scopesList);
        scopesList = NULL;
    }
    return NULL;

}
