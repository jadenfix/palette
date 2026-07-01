#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "connector_tool.h"



static connector_tool_t *connector_tool_create_internal(
    char *description,
    object_t *input_schema,
    char *name,
    int no_auth,
    char *slug,
    list_t *tags,
    char *toolkit
    ) {
    connector_tool_t *connector_tool_local_var = malloc(sizeof(connector_tool_t));
    if (!connector_tool_local_var) {
        return NULL;
    }
    connector_tool_local_var->description = description;
    connector_tool_local_var->input_schema = input_schema;
    connector_tool_local_var->name = name;
    connector_tool_local_var->no_auth = no_auth;
    connector_tool_local_var->slug = slug;
    connector_tool_local_var->tags = tags;
    connector_tool_local_var->toolkit = toolkit;

    connector_tool_local_var->_library_owned = 1;
    return connector_tool_local_var;
}

__attribute__((deprecated)) connector_tool_t *connector_tool_create(
    char *description,
    object_t *input_schema,
    char *name,
    int no_auth,
    char *slug,
    list_t *tags,
    char *toolkit
    ) {
    return connector_tool_create_internal (
        description,
        input_schema,
        name,
        no_auth,
        slug,
        tags,
        toolkit
        );
}

void connector_tool_free(connector_tool_t *connector_tool) {
    if(NULL == connector_tool){
        return ;
    }
    if(connector_tool->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "connector_tool_free");
        return ;
    }
    listEntry_t *listEntry;
    if (connector_tool->description) {
        free(connector_tool->description);
        connector_tool->description = NULL;
    }
    if (connector_tool->input_schema) {
        object_free(connector_tool->input_schema);
        connector_tool->input_schema = NULL;
    }
    if (connector_tool->name) {
        free(connector_tool->name);
        connector_tool->name = NULL;
    }
    if (connector_tool->slug) {
        free(connector_tool->slug);
        connector_tool->slug = NULL;
    }
    if (connector_tool->tags) {
        list_ForEach(listEntry, connector_tool->tags) {
            free(listEntry->data);
        }
        list_freeList(connector_tool->tags);
        connector_tool->tags = NULL;
    }
    if (connector_tool->toolkit) {
        free(connector_tool->toolkit);
        connector_tool->toolkit = NULL;
    }
    free(connector_tool);
}

cJSON *connector_tool_convertToJSON(connector_tool_t *connector_tool) {
    cJSON *item = cJSON_CreateObject();

    // connector_tool->description
    if(connector_tool->description) {
    if(cJSON_AddStringToObject(item, "description", connector_tool->description) == NULL) {
    goto fail; //String
    }
    }


    // connector_tool->input_schema
    if(connector_tool->input_schema) {
    cJSON *input_schema_object = object_convertToJSON(connector_tool->input_schema);
    if(input_schema_object == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "input_schema", input_schema_object);
    if(item->child == NULL) {
    goto fail;
    }
    }


    // connector_tool->name
    if (!connector_tool->name) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "name", connector_tool->name) == NULL) {
    goto fail; //String
    }


    // connector_tool->no_auth
    if(connector_tool->no_auth) {
    if(cJSON_AddBoolToObject(item, "no_auth", connector_tool->no_auth) == NULL) {
    goto fail; //Bool
    }
    }


    // connector_tool->slug
    if (!connector_tool->slug) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "slug", connector_tool->slug) == NULL) {
    goto fail; //String
    }


    // connector_tool->tags
    if(connector_tool->tags) {
    cJSON *tags = cJSON_AddArrayToObject(item, "tags");
    if(tags == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *tagsListEntry;
    list_ForEach(tagsListEntry, connector_tool->tags) {
    if(cJSON_AddStringToObject(tags, "", tagsListEntry->data) == NULL)
    {
        goto fail;
    }
    }
    }


    // connector_tool->toolkit
    if(connector_tool->toolkit) {
    if(cJSON_AddStringToObject(item, "toolkit", connector_tool->toolkit) == NULL) {
    goto fail; //String
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

connector_tool_t *connector_tool_parseFromJSON(cJSON *connector_toolJSON){

    connector_tool_t *connector_tool_local_var = NULL;

    // define the local list for connector_tool->tags
    list_t *tagsList = NULL;

    // connector_tool->description
    cJSON *description = cJSON_GetObjectItemCaseSensitive(connector_toolJSON, "description");
    if (cJSON_IsNull(description)) {
        description = NULL;
    }
    if (description) { 
    if(!cJSON_IsString(description) && !cJSON_IsNull(description))
    {
    goto end; //String
    }
    }

    // connector_tool->input_schema
    cJSON *input_schema = cJSON_GetObjectItemCaseSensitive(connector_toolJSON, "input_schema");
    if (cJSON_IsNull(input_schema)) {
        input_schema = NULL;
    }
    object_t *input_schema_local_object = NULL;
    if (input_schema) { 
    input_schema_local_object = object_parseFromJSON(input_schema); //object
    }

    // connector_tool->name
    cJSON *name = cJSON_GetObjectItemCaseSensitive(connector_toolJSON, "name");
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

    // connector_tool->no_auth
    cJSON *no_auth = cJSON_GetObjectItemCaseSensitive(connector_toolJSON, "no_auth");
    if (cJSON_IsNull(no_auth)) {
        no_auth = NULL;
    }
    if (no_auth) { 
    if(!cJSON_IsBool(no_auth))
    {
    goto end; //Bool
    }
    }

    // connector_tool->slug
    cJSON *slug = cJSON_GetObjectItemCaseSensitive(connector_toolJSON, "slug");
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

    // connector_tool->tags
    cJSON *tags = cJSON_GetObjectItemCaseSensitive(connector_toolJSON, "tags");
    if (cJSON_IsNull(tags)) {
        tags = NULL;
    }
    if (tags) { 
    cJSON *tags_local = NULL;
    if(!cJSON_IsArray(tags)) {
        goto end;//primitive container
    }
    tagsList = list_createList();

    cJSON_ArrayForEach(tags_local, tags)
    {
        if(!cJSON_IsString(tags_local))
        {
            goto end;
        }
        list_addElement(tagsList , strdup(tags_local->valuestring));
    }
    }

    // connector_tool->toolkit
    cJSON *toolkit = cJSON_GetObjectItemCaseSensitive(connector_toolJSON, "toolkit");
    if (cJSON_IsNull(toolkit)) {
        toolkit = NULL;
    }
    if (toolkit) { 
    if(!cJSON_IsString(toolkit) && !cJSON_IsNull(toolkit))
    {
    goto end; //String
    }
    }


    connector_tool_local_var = connector_tool_create_internal (
        description && !cJSON_IsNull(description) ? strdup(description->valuestring) : NULL,
        input_schema ? input_schema_local_object : NULL,
        strdup(name->valuestring),
        no_auth ? no_auth->valueint : 0,
        strdup(slug->valuestring),
        tags ? tagsList : NULL,
        toolkit && !cJSON_IsNull(toolkit) ? strdup(toolkit->valuestring) : NULL
        );

    return connector_tool_local_var;
end:
    if (tagsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, tagsList) {
            free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(tagsList);
        tagsList = NULL;
    }
    return NULL;

}
