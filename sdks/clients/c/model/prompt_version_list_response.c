#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt_version_list_response.h"



static prompt_version_list_response_t *prompt_version_list_response_create_internal(
    list_t *versions
    ) {
    prompt_version_list_response_t *prompt_version_list_response_local_var = malloc(sizeof(prompt_version_list_response_t));
    if (!prompt_version_list_response_local_var) {
        return NULL;
    }
    prompt_version_list_response_local_var->versions = versions;

    prompt_version_list_response_local_var->_library_owned = 1;
    return prompt_version_list_response_local_var;
}

__attribute__((deprecated)) prompt_version_list_response_t *prompt_version_list_response_create(
    list_t *versions
    ) {
    return prompt_version_list_response_create_internal (
        versions
        );
}

void prompt_version_list_response_free(prompt_version_list_response_t *prompt_version_list_response) {
    if(NULL == prompt_version_list_response){
        return ;
    }
    if(prompt_version_list_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_version_list_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt_version_list_response->versions) {
        list_ForEach(listEntry, prompt_version_list_response->versions) {
            prompt_version_free(listEntry->data);
        }
        list_freeList(prompt_version_list_response->versions);
        prompt_version_list_response->versions = NULL;
    }
    free(prompt_version_list_response);
}

cJSON *prompt_version_list_response_convertToJSON(prompt_version_list_response_t *prompt_version_list_response) {
    cJSON *item = cJSON_CreateObject();

    // prompt_version_list_response->versions
    if (!prompt_version_list_response->versions) {
        goto fail;
    }
    cJSON *versions = cJSON_AddArrayToObject(item, "versions");
    if(versions == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *versionsListEntry;
    if (prompt_version_list_response->versions) {
    list_ForEach(versionsListEntry, prompt_version_list_response->versions) {
    cJSON *itemLocal = prompt_version_convertToJSON(versionsListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(versions, itemLocal);
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_version_list_response_t *prompt_version_list_response_parseFromJSON(cJSON *prompt_version_list_responseJSON){

    prompt_version_list_response_t *prompt_version_list_response_local_var = NULL;

    // define the local list for prompt_version_list_response->versions
    list_t *versionsList = NULL;

    // prompt_version_list_response->versions
    cJSON *versions = cJSON_GetObjectItemCaseSensitive(prompt_version_list_responseJSON, "versions");
    if (cJSON_IsNull(versions)) {
        versions = NULL;
    }
    if (!versions) {
        goto end;
    }

    
    cJSON *versions_local_nonprimitive = NULL;
    if(!cJSON_IsArray(versions)){
        goto end; //nonprimitive container
    }

    versionsList = list_createList();

    cJSON_ArrayForEach(versions_local_nonprimitive,versions )
    {
        if(!cJSON_IsObject(versions_local_nonprimitive)){
            goto end;
        }
        prompt_version_t *versionsItem = prompt_version_parseFromJSON(versions_local_nonprimitive);

        list_addElement(versionsList, versionsItem);
    }


    prompt_version_list_response_local_var = prompt_version_list_response_create_internal (
        versionsList
        );

    return prompt_version_list_response_local_var;
end:
    if (versionsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, versionsList) {
            prompt_version_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(versionsList);
        versionsList = NULL;
    }
    return NULL;

}
