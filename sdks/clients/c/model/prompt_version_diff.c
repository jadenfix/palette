#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt_version_diff.h"



static prompt_version_diff_t *prompt_version_diff_create_internal(
    char *from_version_id,
    list_t *lines,
    char *to_version_id
    ) {
    prompt_version_diff_t *prompt_version_diff_local_var = malloc(sizeof(prompt_version_diff_t));
    if (!prompt_version_diff_local_var) {
        return NULL;
    }
    prompt_version_diff_local_var->from_version_id = from_version_id;
    prompt_version_diff_local_var->lines = lines;
    prompt_version_diff_local_var->to_version_id = to_version_id;

    prompt_version_diff_local_var->_library_owned = 1;
    return prompt_version_diff_local_var;
}

__attribute__((deprecated)) prompt_version_diff_t *prompt_version_diff_create(
    char *from_version_id,
    list_t *lines,
    char *to_version_id
    ) {
    return prompt_version_diff_create_internal (
        from_version_id,
        lines,
        to_version_id
        );
}

void prompt_version_diff_free(prompt_version_diff_t *prompt_version_diff) {
    if(NULL == prompt_version_diff){
        return ;
    }
    if(prompt_version_diff->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_version_diff_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt_version_diff->from_version_id) {
        free(prompt_version_diff->from_version_id);
        prompt_version_diff->from_version_id = NULL;
    }
    if (prompt_version_diff->lines) {
        list_ForEach(listEntry, prompt_version_diff->lines) {
            diff_line_free(listEntry->data);
        }
        list_freeList(prompt_version_diff->lines);
        prompt_version_diff->lines = NULL;
    }
    if (prompt_version_diff->to_version_id) {
        free(prompt_version_diff->to_version_id);
        prompt_version_diff->to_version_id = NULL;
    }
    free(prompt_version_diff);
}

cJSON *prompt_version_diff_convertToJSON(prompt_version_diff_t *prompt_version_diff) {
    cJSON *item = cJSON_CreateObject();

    // prompt_version_diff->from_version_id
    if (!prompt_version_diff->from_version_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "from_version_id", prompt_version_diff->from_version_id) == NULL) {
    goto fail; //String
    }


    // prompt_version_diff->lines
    if (!prompt_version_diff->lines) {
        goto fail;
    }
    cJSON *lines = cJSON_AddArrayToObject(item, "lines");
    if(lines == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *linesListEntry;
    if (prompt_version_diff->lines) {
    list_ForEach(linesListEntry, prompt_version_diff->lines) {
    cJSON *itemLocal = diff_line_convertToJSON(linesListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(lines, itemLocal);
    }
    }


    // prompt_version_diff->to_version_id
    if (!prompt_version_diff->to_version_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "to_version_id", prompt_version_diff->to_version_id) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_version_diff_t *prompt_version_diff_parseFromJSON(cJSON *prompt_version_diffJSON){

    prompt_version_diff_t *prompt_version_diff_local_var = NULL;

    // define the local list for prompt_version_diff->lines
    list_t *linesList = NULL;

    // prompt_version_diff->from_version_id
    cJSON *from_version_id = cJSON_GetObjectItemCaseSensitive(prompt_version_diffJSON, "from_version_id");
    if (cJSON_IsNull(from_version_id)) {
        from_version_id = NULL;
    }
    if (!from_version_id) {
        goto end;
    }

    
    if(!cJSON_IsString(from_version_id))
    {
    goto end; //String
    }

    // prompt_version_diff->lines
    cJSON *lines = cJSON_GetObjectItemCaseSensitive(prompt_version_diffJSON, "lines");
    if (cJSON_IsNull(lines)) {
        lines = NULL;
    }
    if (!lines) {
        goto end;
    }

    
    cJSON *lines_local_nonprimitive = NULL;
    if(!cJSON_IsArray(lines)){
        goto end; //nonprimitive container
    }

    linesList = list_createList();

    cJSON_ArrayForEach(lines_local_nonprimitive,lines )
    {
        if(!cJSON_IsObject(lines_local_nonprimitive)){
            goto end;
        }
        diff_line_t *linesItem = diff_line_parseFromJSON(lines_local_nonprimitive);

        list_addElement(linesList, linesItem);
    }

    // prompt_version_diff->to_version_id
    cJSON *to_version_id = cJSON_GetObjectItemCaseSensitive(prompt_version_diffJSON, "to_version_id");
    if (cJSON_IsNull(to_version_id)) {
        to_version_id = NULL;
    }
    if (!to_version_id) {
        goto end;
    }

    
    if(!cJSON_IsString(to_version_id))
    {
    goto end; //String
    }


    prompt_version_diff_local_var = prompt_version_diff_create_internal (
        strdup(from_version_id->valuestring),
        linesList,
        strdup(to_version_id->valuestring)
        );

    return prompt_version_diff_local_var;
end:
    if (linesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, linesList) {
            diff_line_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(linesList);
        linesList = NULL;
    }
    return NULL;

}
