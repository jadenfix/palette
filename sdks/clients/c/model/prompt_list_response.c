#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt_list_response.h"



static prompt_list_response_t *prompt_list_response_create_internal(
    list_t *prompts
    ) {
    prompt_list_response_t *prompt_list_response_local_var = malloc(sizeof(prompt_list_response_t));
    if (!prompt_list_response_local_var) {
        return NULL;
    }
    prompt_list_response_local_var->prompts = prompts;

    prompt_list_response_local_var->_library_owned = 1;
    return prompt_list_response_local_var;
}

__attribute__((deprecated)) prompt_list_response_t *prompt_list_response_create(
    list_t *prompts
    ) {
    return prompt_list_response_create_internal (
        prompts
        );
}

void prompt_list_response_free(prompt_list_response_t *prompt_list_response) {
    if(NULL == prompt_list_response){
        return ;
    }
    if(prompt_list_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_list_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt_list_response->prompts) {
        list_ForEach(listEntry, prompt_list_response->prompts) {
            prompt_free(listEntry->data);
        }
        list_freeList(prompt_list_response->prompts);
        prompt_list_response->prompts = NULL;
    }
    free(prompt_list_response);
}

cJSON *prompt_list_response_convertToJSON(prompt_list_response_t *prompt_list_response) {
    cJSON *item = cJSON_CreateObject();

    // prompt_list_response->prompts
    if (!prompt_list_response->prompts) {
        goto fail;
    }
    cJSON *prompts = cJSON_AddArrayToObject(item, "prompts");
    if(prompts == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *promptsListEntry;
    if (prompt_list_response->prompts) {
    list_ForEach(promptsListEntry, prompt_list_response->prompts) {
    cJSON *itemLocal = prompt_convertToJSON(promptsListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(prompts, itemLocal);
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_list_response_t *prompt_list_response_parseFromJSON(cJSON *prompt_list_responseJSON){

    prompt_list_response_t *prompt_list_response_local_var = NULL;

    // define the local list for prompt_list_response->prompts
    list_t *promptsList = NULL;

    // prompt_list_response->prompts
    cJSON *prompts = cJSON_GetObjectItemCaseSensitive(prompt_list_responseJSON, "prompts");
    if (cJSON_IsNull(prompts)) {
        prompts = NULL;
    }
    if (!prompts) {
        goto end;
    }

    
    cJSON *prompts_local_nonprimitive = NULL;
    if(!cJSON_IsArray(prompts)){
        goto end; //nonprimitive container
    }

    promptsList = list_createList();

    cJSON_ArrayForEach(prompts_local_nonprimitive,prompts )
    {
        if(!cJSON_IsObject(prompts_local_nonprimitive)){
            goto end;
        }
        prompt_t *promptsItem = prompt_parseFromJSON(prompts_local_nonprimitive);

        list_addElement(promptsList, promptsItem);
    }


    prompt_list_response_local_var = prompt_list_response_create_internal (
        promptsList
        );

    return prompt_list_response_local_var;
end:
    if (promptsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, promptsList) {
            prompt_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(promptsList);
        promptsList = NULL;
    }
    return NULL;

}
