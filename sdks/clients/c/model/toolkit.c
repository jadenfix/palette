#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "toolkit.h"



static toolkit_t *toolkit_create_internal(
    list_t *auth_schemes,
    char *description,
    char *name,
    int no_auth,
    char *slug,
    int tools_count
    ) {
    toolkit_t *toolkit_local_var = malloc(sizeof(toolkit_t));
    if (!toolkit_local_var) {
        return NULL;
    }
    toolkit_local_var->auth_schemes = auth_schemes;
    toolkit_local_var->description = description;
    toolkit_local_var->name = name;
    toolkit_local_var->no_auth = no_auth;
    toolkit_local_var->slug = slug;
    toolkit_local_var->tools_count = tools_count;

    toolkit_local_var->_library_owned = 1;
    return toolkit_local_var;
}

__attribute__((deprecated)) toolkit_t *toolkit_create(
    list_t *auth_schemes,
    char *description,
    char *name,
    int no_auth,
    char *slug,
    int tools_count
    ) {
    return toolkit_create_internal (
        auth_schemes,
        description,
        name,
        no_auth,
        slug,
        tools_count
        );
}

void toolkit_free(toolkit_t *toolkit) {
    if(NULL == toolkit){
        return ;
    }
    if(toolkit->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "toolkit_free");
        return ;
    }
    listEntry_t *listEntry;
    if (toolkit->auth_schemes) {
        list_ForEach(listEntry, toolkit->auth_schemes) {
            free(listEntry->data);
        }
        list_freeList(toolkit->auth_schemes);
        toolkit->auth_schemes = NULL;
    }
    if (toolkit->description) {
        free(toolkit->description);
        toolkit->description = NULL;
    }
    if (toolkit->name) {
        free(toolkit->name);
        toolkit->name = NULL;
    }
    if (toolkit->slug) {
        free(toolkit->slug);
        toolkit->slug = NULL;
    }
    free(toolkit);
}

cJSON *toolkit_convertToJSON(toolkit_t *toolkit) {
    cJSON *item = cJSON_CreateObject();

    // toolkit->auth_schemes
    if(toolkit->auth_schemes) {
    cJSON *auth_schemes = cJSON_AddArrayToObject(item, "auth_schemes");
    if(auth_schemes == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *auth_schemesListEntry;
    list_ForEach(auth_schemesListEntry, toolkit->auth_schemes) {
    if(cJSON_AddStringToObject(auth_schemes, "", auth_schemesListEntry->data) == NULL)
    {
        goto fail;
    }
    }
    }


    // toolkit->description
    if(toolkit->description) {
    if(cJSON_AddStringToObject(item, "description", toolkit->description) == NULL) {
    goto fail; //String
    }
    }


    // toolkit->name
    if (!toolkit->name) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "name", toolkit->name) == NULL) {
    goto fail; //String
    }


    // toolkit->no_auth
    if(toolkit->no_auth) {
    if(cJSON_AddBoolToObject(item, "no_auth", toolkit->no_auth) == NULL) {
    goto fail; //Bool
    }
    }


    // toolkit->slug
    if (!toolkit->slug) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "slug", toolkit->slug) == NULL) {
    goto fail; //String
    }


    // toolkit->tools_count
    if(toolkit->tools_count) {
    if(cJSON_AddNumberToObject(item, "tools_count", toolkit->tools_count) == NULL) {
    goto fail; //Numeric
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

toolkit_t *toolkit_parseFromJSON(cJSON *toolkitJSON){

    toolkit_t *toolkit_local_var = NULL;

    // define the local list for toolkit->auth_schemes
    list_t *auth_schemesList = NULL;

    // toolkit->auth_schemes
    cJSON *auth_schemes = cJSON_GetObjectItemCaseSensitive(toolkitJSON, "auth_schemes");
    if (cJSON_IsNull(auth_schemes)) {
        auth_schemes = NULL;
    }
    if (auth_schemes) { 
    cJSON *auth_schemes_local = NULL;
    if(!cJSON_IsArray(auth_schemes)) {
        goto end;//primitive container
    }
    auth_schemesList = list_createList();

    cJSON_ArrayForEach(auth_schemes_local, auth_schemes)
    {
        if(!cJSON_IsString(auth_schemes_local))
        {
            goto end;
        }
        list_addElement(auth_schemesList , strdup(auth_schemes_local->valuestring));
    }
    }

    // toolkit->description
    cJSON *description = cJSON_GetObjectItemCaseSensitive(toolkitJSON, "description");
    if (cJSON_IsNull(description)) {
        description = NULL;
    }
    if (description) { 
    if(!cJSON_IsString(description) && !cJSON_IsNull(description))
    {
    goto end; //String
    }
    }

    // toolkit->name
    cJSON *name = cJSON_GetObjectItemCaseSensitive(toolkitJSON, "name");
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

    // toolkit->no_auth
    cJSON *no_auth = cJSON_GetObjectItemCaseSensitive(toolkitJSON, "no_auth");
    if (cJSON_IsNull(no_auth)) {
        no_auth = NULL;
    }
    if (no_auth) { 
    if(!cJSON_IsBool(no_auth))
    {
    goto end; //Bool
    }
    }

    // toolkit->slug
    cJSON *slug = cJSON_GetObjectItemCaseSensitive(toolkitJSON, "slug");
    if (cJSON_IsNull(slug)) {
        slug = NULL;
    }
    if (!slug) {
        goto end;
    }

    
    if(!cJSON_IsString(slug))
    {
    goto end; //String
    }

    // toolkit->tools_count
    cJSON *tools_count = cJSON_GetObjectItemCaseSensitive(toolkitJSON, "tools_count");
    if (cJSON_IsNull(tools_count)) {
        tools_count = NULL;
    }
    if (tools_count) { 
    if(!cJSON_IsNumber(tools_count))
    {
    goto end; //Numeric
    }
    }


    toolkit_local_var = toolkit_create_internal (
        auth_schemes ? auth_schemesList : NULL,
        description && !cJSON_IsNull(description) ? strdup(description->valuestring) : NULL,
        strdup(name->valuestring),
        no_auth ? no_auth->valueint : 0,
        strdup(slug->valuestring),
        tools_count ? tools_count->valuedouble : 0
        );

    return toolkit_local_var;
end:
    if (auth_schemesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, auth_schemesList) {
            free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(auth_schemesList);
        auth_schemesList = NULL;
    }
    return NULL;

}
