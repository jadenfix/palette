#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "chat_completion_response.h"



static chat_completion_response_t *chat_completion_response_create_internal(
    list_t *choices,
    long created,
    char *id,
    char *model,
    char *object,
    chat_completion_usage_t *usage
    ) {
    chat_completion_response_t *chat_completion_response_local_var = malloc(sizeof(chat_completion_response_t));
    if (!chat_completion_response_local_var) {
        return NULL;
    }
    chat_completion_response_local_var->choices = choices;
    chat_completion_response_local_var->created = created;
    chat_completion_response_local_var->id = id;
    chat_completion_response_local_var->model = model;
    chat_completion_response_local_var->object = object;
    chat_completion_response_local_var->usage = usage;

    chat_completion_response_local_var->_library_owned = 1;
    return chat_completion_response_local_var;
}

__attribute__((deprecated)) chat_completion_response_t *chat_completion_response_create(
    list_t *choices,
    long created,
    char *id,
    char *model,
    char *object,
    chat_completion_usage_t *usage
    ) {
    return chat_completion_response_create_internal (
        choices,
        created,
        id,
        model,
        object,
        usage
        );
}

void chat_completion_response_free(chat_completion_response_t *chat_completion_response) {
    if(NULL == chat_completion_response){
        return ;
    }
    if(chat_completion_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "chat_completion_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (chat_completion_response->choices) {
        list_ForEach(listEntry, chat_completion_response->choices) {
            chat_completion_choice_free(listEntry->data);
        }
        list_freeList(chat_completion_response->choices);
        chat_completion_response->choices = NULL;
    }
    if (chat_completion_response->id) {
        free(chat_completion_response->id);
        chat_completion_response->id = NULL;
    }
    if (chat_completion_response->model) {
        free(chat_completion_response->model);
        chat_completion_response->model = NULL;
    }
    if (chat_completion_response->object) {
        free(chat_completion_response->object);
        chat_completion_response->object = NULL;
    }
    if (chat_completion_response->usage) {
        chat_completion_usage_free(chat_completion_response->usage);
        chat_completion_response->usage = NULL;
    }
    free(chat_completion_response);
}

cJSON *chat_completion_response_convertToJSON(chat_completion_response_t *chat_completion_response) {
    cJSON *item = cJSON_CreateObject();

    // chat_completion_response->choices
    if (!chat_completion_response->choices) {
        goto fail;
    }
    cJSON *choices = cJSON_AddArrayToObject(item, "choices");
    if(choices == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *choicesListEntry;
    if (chat_completion_response->choices) {
    list_ForEach(choicesListEntry, chat_completion_response->choices) {
    cJSON *itemLocal = chat_completion_choice_convertToJSON(choicesListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(choices, itemLocal);
    }
    }


    // chat_completion_response->created
    if (!chat_completion_response->created) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "created", chat_completion_response->created) == NULL) {
    goto fail; //Numeric
    }


    // chat_completion_response->id
    if (!chat_completion_response->id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "id", chat_completion_response->id) == NULL) {
    goto fail; //String
    }


    // chat_completion_response->model
    if (!chat_completion_response->model) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "model", chat_completion_response->model) == NULL) {
    goto fail; //String
    }


    // chat_completion_response->object
    if (!chat_completion_response->object) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "object", chat_completion_response->object) == NULL) {
    goto fail; //String
    }


    // chat_completion_response->usage
    if (!chat_completion_response->usage) {
        goto fail;
    }
    cJSON *usage_local_JSON = chat_completion_usage_convertToJSON(chat_completion_response->usage);
    if(usage_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "usage", usage_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

chat_completion_response_t *chat_completion_response_parseFromJSON(cJSON *chat_completion_responseJSON){

    chat_completion_response_t *chat_completion_response_local_var = NULL;

    // define the local list for chat_completion_response->choices
    list_t *choicesList = NULL;

    // define the local variable for chat_completion_response->usage
    chat_completion_usage_t *usage_local_nonprim = NULL;

    // chat_completion_response->choices
    cJSON *choices = cJSON_GetObjectItemCaseSensitive(chat_completion_responseJSON, "choices");
    if (cJSON_IsNull(choices)) {
        choices = NULL;
    }
    if (!choices) {
        goto end;
    }

    
    cJSON *choices_local_nonprimitive = NULL;
    if(!cJSON_IsArray(choices)){
        goto end; //nonprimitive container
    }

    choicesList = list_createList();

    cJSON_ArrayForEach(choices_local_nonprimitive,choices )
    {
        if(!cJSON_IsObject(choices_local_nonprimitive)){
            goto end;
        }
        chat_completion_choice_t *choicesItem = chat_completion_choice_parseFromJSON(choices_local_nonprimitive);

        list_addElement(choicesList, choicesItem);
    }

    // chat_completion_response->created
    cJSON *created = cJSON_GetObjectItemCaseSensitive(chat_completion_responseJSON, "created");
    if (cJSON_IsNull(created)) {
        created = NULL;
    }
    if (!created) {
        goto end;
    }

    
    if(!cJSON_IsNumber(created))
    {
    goto end; //Numeric
    }

    // chat_completion_response->id
    cJSON *id = cJSON_GetObjectItemCaseSensitive(chat_completion_responseJSON, "id");
    if (cJSON_IsNull(id)) {
        id = NULL;
    }
    if (!id) {
        goto end;
    }

    
    if(!cJSON_IsString(id))
    {
    goto end; //String
    }

    // chat_completion_response->model
    cJSON *model = cJSON_GetObjectItemCaseSensitive(chat_completion_responseJSON, "model");
    if (cJSON_IsNull(model)) {
        model = NULL;
    }
    if (!model) {
        goto end;
    }

    
    if(!cJSON_IsString(model))
    {
    goto end; //String
    }

    // chat_completion_response->object
    cJSON *object = cJSON_GetObjectItemCaseSensitive(chat_completion_responseJSON, "object");
    if (cJSON_IsNull(object)) {
        object = NULL;
    }
    if (!object) {
        goto end;
    }

    
    if(!cJSON_IsString(object))
    {
    goto end; //String
    }

    // chat_completion_response->usage
    cJSON *usage = cJSON_GetObjectItemCaseSensitive(chat_completion_responseJSON, "usage");
    if (cJSON_IsNull(usage)) {
        usage = NULL;
    }
    if (!usage) {
        goto end;
    }

    
    usage_local_nonprim = chat_completion_usage_parseFromJSON(usage); //nonprimitive


    chat_completion_response_local_var = chat_completion_response_create_internal (
        choicesList,
        created->valuedouble,
        strdup(id->valuestring),
        strdup(model->valuestring),
        strdup(object->valuestring),
        usage_local_nonprim
        );

    return chat_completion_response_local_var;
end:
    if (choicesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, choicesList) {
            chat_completion_choice_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(choicesList);
        choicesList = NULL;
    }
    if (usage_local_nonprim) {
        chat_completion_usage_free(usage_local_nonprim);
        usage_local_nonprim = NULL;
    }
    return NULL;

}
