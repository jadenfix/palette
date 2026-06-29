#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "prompt_template.h"



static prompt_template_t *prompt_template_create_internal(
    char *body,
    list_t *tags,
    list_t *variables
    ) {
    prompt_template_t *prompt_template_local_var = malloc(sizeof(prompt_template_t));
    if (!prompt_template_local_var) {
        return NULL;
    }
    prompt_template_local_var->body = body;
    prompt_template_local_var->tags = tags;
    prompt_template_local_var->variables = variables;

    prompt_template_local_var->_library_owned = 1;
    return prompt_template_local_var;
}

__attribute__((deprecated)) prompt_template_t *prompt_template_create(
    char *body,
    list_t *tags,
    list_t *variables
    ) {
    return prompt_template_create_internal (
        body,
        tags,
        variables
        );
}

void prompt_template_free(prompt_template_t *prompt_template) {
    if(NULL == prompt_template){
        return ;
    }
    if(prompt_template->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "prompt_template_free");
        return ;
    }
    listEntry_t *listEntry;
    if (prompt_template->body) {
        free(prompt_template->body);
        prompt_template->body = NULL;
    }
    if (prompt_template->tags) {
        list_ForEach(listEntry, prompt_template->tags) {
            free(listEntry->data);
        }
        list_freeList(prompt_template->tags);
        prompt_template->tags = NULL;
    }
    if (prompt_template->variables) {
        list_ForEach(listEntry, prompt_template->variables) {
            prompt_variable_free(listEntry->data);
        }
        list_freeList(prompt_template->variables);
        prompt_template->variables = NULL;
    }
    free(prompt_template);
}

cJSON *prompt_template_convertToJSON(prompt_template_t *prompt_template) {
    cJSON *item = cJSON_CreateObject();

    // prompt_template->body
    if (!prompt_template->body) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "body", prompt_template->body) == NULL) {
    goto fail; //String
    }


    // prompt_template->tags
    if (!prompt_template->tags) {
        goto fail;
    }
    cJSON *tags = cJSON_AddArrayToObject(item, "tags");
    if(tags == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *tagsListEntry;
    list_ForEach(tagsListEntry, prompt_template->tags) {
    if(cJSON_AddStringToObject(tags, "", tagsListEntry->data) == NULL)
    {
        goto fail;
    }
    }


    // prompt_template->variables
    if (!prompt_template->variables) {
        goto fail;
    }
    cJSON *variables = cJSON_AddArrayToObject(item, "variables");
    if(variables == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *variablesListEntry;
    if (prompt_template->variables) {
    list_ForEach(variablesListEntry, prompt_template->variables) {
    cJSON *itemLocal = prompt_variable_convertToJSON(variablesListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(variables, itemLocal);
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

prompt_template_t *prompt_template_parseFromJSON(cJSON *prompt_templateJSON){

    prompt_template_t *prompt_template_local_var = NULL;

    // define the local list for prompt_template->tags
    list_t *tagsList = NULL;

    // define the local list for prompt_template->variables
    list_t *variablesList = NULL;

    // prompt_template->body
    cJSON *body = cJSON_GetObjectItemCaseSensitive(prompt_templateJSON, "body");
    if (cJSON_IsNull(body)) {
        body = NULL;
    }
    if (!body) {
        goto end;
    }

    
    if(!cJSON_IsString(body))
    {
    goto end; //String
    }

    // prompt_template->tags
    cJSON *tags = cJSON_GetObjectItemCaseSensitive(prompt_templateJSON, "tags");
    if (cJSON_IsNull(tags)) {
        tags = NULL;
    }
    if (!tags) {
        goto end;
    }

    
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

    // prompt_template->variables
    cJSON *variables = cJSON_GetObjectItemCaseSensitive(prompt_templateJSON, "variables");
    if (cJSON_IsNull(variables)) {
        variables = NULL;
    }
    if (!variables) {
        goto end;
    }

    
    cJSON *variables_local_nonprimitive = NULL;
    if(!cJSON_IsArray(variables)){
        goto end; //nonprimitive container
    }

    variablesList = list_createList();

    cJSON_ArrayForEach(variables_local_nonprimitive,variables )
    {
        if(!cJSON_IsObject(variables_local_nonprimitive)){
            goto end;
        }
        prompt_variable_t *variablesItem = prompt_variable_parseFromJSON(variables_local_nonprimitive);

        list_addElement(variablesList, variablesItem);
    }


    prompt_template_local_var = prompt_template_create_internal (
        strdup(body->valuestring),
        tagsList,
        variablesList
        );

    return prompt_template_local_var;
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
    if (variablesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, variablesList) {
            prompt_variable_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(variablesList);
        variablesList = NULL;
    }
    return NULL;

}
