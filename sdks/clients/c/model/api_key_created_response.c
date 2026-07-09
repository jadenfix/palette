#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "api_key_created_response.h"


char* api_key_created_response_scopes_ToString(beater_api_api_key_created_response__e scopes) {
    char *scopesArray[] =  { "NULL", "trace:write", "trace:read", "dataset:write", "scenario:write", "scenario:read", "eval:run", "pii:unmask", "admin" };
    return scopesArray[scopes - 1];
}

beater_api_api_key_created_response__e api_key_created_response_scopes_FromString(char* scopes) {
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

static api_key_created_response_t *api_key_created_response_create_internal(
    int active,
    char *api_key_id,
    char *created_at,
    char *environment_id,
    char *project_id,
    list_t *scopes,
    char *secret,
    char *tenant_id
    ) {
    api_key_created_response_t *api_key_created_response_local_var = malloc(sizeof(api_key_created_response_t));
    if (!api_key_created_response_local_var) {
        return NULL;
    }
    api_key_created_response_local_var->active = active;
    api_key_created_response_local_var->api_key_id = api_key_id;
    api_key_created_response_local_var->created_at = created_at;
    api_key_created_response_local_var->environment_id = environment_id;
    api_key_created_response_local_var->project_id = project_id;
    api_key_created_response_local_var->scopes = scopes;
    api_key_created_response_local_var->secret = secret;
    api_key_created_response_local_var->tenant_id = tenant_id;

    api_key_created_response_local_var->_library_owned = 1;
    return api_key_created_response_local_var;
}

__attribute__((deprecated)) api_key_created_response_t *api_key_created_response_create(
    int active,
    char *api_key_id,
    char *created_at,
    char *environment_id,
    char *project_id,
    list_t *scopes,
    char *secret,
    char *tenant_id
    ) {
    return api_key_created_response_create_internal (
        active,
        api_key_id,
        created_at,
        environment_id,
        project_id,
        scopes,
        secret,
        tenant_id
        );
}

void api_key_created_response_free(api_key_created_response_t *api_key_created_response) {
    if(NULL == api_key_created_response){
        return ;
    }
    if(api_key_created_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "api_key_created_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (api_key_created_response->api_key_id) {
        free(api_key_created_response->api_key_id);
        api_key_created_response->api_key_id = NULL;
    }
    if (api_key_created_response->created_at) {
        free(api_key_created_response->created_at);
        api_key_created_response->created_at = NULL;
    }
    if (api_key_created_response->environment_id) {
        free(api_key_created_response->environment_id);
        api_key_created_response->environment_id = NULL;
    }
    if (api_key_created_response->project_id) {
        free(api_key_created_response->project_id);
        api_key_created_response->project_id = NULL;
    }
    if (api_key_created_response->scopes) {
        list_ForEach(listEntry, api_key_created_response->scopes) {
            api_scope_free(listEntry->data);
        }
        list_freeList(api_key_created_response->scopes);
        api_key_created_response->scopes = NULL;
    }
    if (api_key_created_response->secret) {
        free(api_key_created_response->secret);
        api_key_created_response->secret = NULL;
    }
    if (api_key_created_response->tenant_id) {
        free(api_key_created_response->tenant_id);
        api_key_created_response->tenant_id = NULL;
    }
    free(api_key_created_response);
}

cJSON *api_key_created_response_convertToJSON(api_key_created_response_t *api_key_created_response) {
    cJSON *item = cJSON_CreateObject();

    // api_key_created_response->active
    if (!api_key_created_response->active) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "active", api_key_created_response->active) == NULL) {
    goto fail; //Bool
    }


    // api_key_created_response->api_key_id
    if (!api_key_created_response->api_key_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "api_key_id", api_key_created_response->api_key_id) == NULL) {
    goto fail; //String
    }


    // api_key_created_response->created_at
    if (!api_key_created_response->created_at) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "created_at", api_key_created_response->created_at) == NULL) {
    goto fail; //Date-Time
    }


    // api_key_created_response->environment_id
    if (!api_key_created_response->environment_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "environment_id", api_key_created_response->environment_id) == NULL) {
    goto fail; //String
    }


    // api_key_created_response->project_id
    if (!api_key_created_response->project_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "project_id", api_key_created_response->project_id) == NULL) {
    goto fail; //String
    }


    // api_key_created_response->scopes
    if (beater_api_list_SCOPES_NULL == api_key_created_response->scopes) {
        goto fail;
    }
    cJSON *scopes = cJSON_AddArrayToObject(item, "scopes");
    if(scopes == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *scopesListEntry;
    if (api_key_created_response->scopes) {
    list_ForEach(scopesListEntry, api_key_created_response->scopes) {
    cJSON *itemLocal = api_scope_convertToJSON((beater_api_api_key_created_response__e)scopesListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(scopes, itemLocal);
    }
    }


    // api_key_created_response->secret
    if (!api_key_created_response->secret) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "secret", api_key_created_response->secret) == NULL) {
    goto fail; //String
    }


    // api_key_created_response->tenant_id
    if (!api_key_created_response->tenant_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "tenant_id", api_key_created_response->tenant_id) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

api_key_created_response_t *api_key_created_response_parseFromJSON(cJSON *api_key_created_responseJSON){

    api_key_created_response_t *api_key_created_response_local_var = NULL;

    // define the local list for api_key_created_response->scopes
    list_t *scopesList = NULL;

    // api_key_created_response->active
    cJSON *active = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "active");
    if (cJSON_IsNull(active)) {
        active = NULL;
    }
    if (!active) {
        goto end;
    }

    
    if(!cJSON_IsBool(active))
    {
    goto end; //Bool
    }

    // api_key_created_response->api_key_id
    cJSON *api_key_id = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "api_key_id");
    if (cJSON_IsNull(api_key_id)) {
        api_key_id = NULL;
    }
    if (!api_key_id) {
        goto end;
    }

    
    if(!cJSON_IsString(api_key_id))
    {
    goto end; //String
    }

    // api_key_created_response->created_at
    cJSON *created_at = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "created_at");
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

    // api_key_created_response->environment_id
    cJSON *environment_id = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "environment_id");
    if (cJSON_IsNull(environment_id)) {
        environment_id = NULL;
    }
    if (!environment_id) {
        goto end;
    }

    
    if(!cJSON_IsString(environment_id))
    {
    goto end; //String
    }

    // api_key_created_response->project_id
    cJSON *project_id = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "project_id");
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

    // api_key_created_response->scopes
    cJSON *scopes = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "scopes");
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
        api_key_created_response_api_scope_e scopesItem = api_scope_parseFromJSON(scopes_local_nonprimitive);

        list_addElement(scopesList, (void *)scopesItem);
    }

    // api_key_created_response->secret
    cJSON *secret = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "secret");
    if (cJSON_IsNull(secret)) {
        secret = NULL;
    }
    if (!secret) {
        goto end;
    }

    
    if(!cJSON_IsString(secret))
    {
    goto end; //String
    }

    // api_key_created_response->tenant_id
    cJSON *tenant_id = cJSON_GetObjectItemCaseSensitive(api_key_created_responseJSON, "tenant_id");
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


    api_key_created_response_local_var = api_key_created_response_create_internal (
        active->valueint,
        strdup(api_key_id->valuestring),
        strdup(created_at->valuestring),
        strdup(environment_id->valuestring),
        strdup(project_id->valuestring),
        scopesList,
        strdup(secret->valuestring),
        strdup(tenant_id->valuestring)
        );

    return api_key_created_response_local_var;
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
