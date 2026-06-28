#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "chat_completion_request.h"



static chat_completion_request_t *chat_completion_request_create_internal(
    long max_tokens,
    list_t *messages,
    char *model,
    char *reasoning_effort,
    double temperature,
    double top_p
    ) {
    chat_completion_request_t *chat_completion_request_local_var = malloc(sizeof(chat_completion_request_t));
    if (!chat_completion_request_local_var) {
        return NULL;
    }
    chat_completion_request_local_var->max_tokens = max_tokens;
    chat_completion_request_local_var->messages = messages;
    chat_completion_request_local_var->model = model;
    chat_completion_request_local_var->reasoning_effort = reasoning_effort;
    chat_completion_request_local_var->temperature = temperature;
    chat_completion_request_local_var->top_p = top_p;

    chat_completion_request_local_var->_library_owned = 1;
    return chat_completion_request_local_var;
}

__attribute__((deprecated)) chat_completion_request_t *chat_completion_request_create(
    long max_tokens,
    list_t *messages,
    char *model,
    char *reasoning_effort,
    double temperature,
    double top_p
    ) {
    return chat_completion_request_create_internal (
        max_tokens,
        messages,
        model,
        reasoning_effort,
        temperature,
        top_p
        );
}

void chat_completion_request_free(chat_completion_request_t *chat_completion_request) {
    if(NULL == chat_completion_request){
        return ;
    }
    if(chat_completion_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "chat_completion_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (chat_completion_request->messages) {
        list_ForEach(listEntry, chat_completion_request->messages) {
            chat_message_free(listEntry->data);
        }
        list_freeList(chat_completion_request->messages);
        chat_completion_request->messages = NULL;
    }
    if (chat_completion_request->model) {
        free(chat_completion_request->model);
        chat_completion_request->model = NULL;
    }
    if (chat_completion_request->reasoning_effort) {
        free(chat_completion_request->reasoning_effort);
        chat_completion_request->reasoning_effort = NULL;
    }
    free(chat_completion_request);
}

cJSON *chat_completion_request_convertToJSON(chat_completion_request_t *chat_completion_request) {
    cJSON *item = cJSON_CreateObject();

    // chat_completion_request->max_tokens
    if(chat_completion_request->max_tokens) {
    if(cJSON_AddNumberToObject(item, "max_tokens", chat_completion_request->max_tokens) == NULL) {
    goto fail; //Numeric
    }
    }


    // chat_completion_request->messages
    if (!chat_completion_request->messages) {
        goto fail;
    }
    cJSON *messages = cJSON_AddArrayToObject(item, "messages");
    if(messages == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *messagesListEntry;
    if (chat_completion_request->messages) {
    list_ForEach(messagesListEntry, chat_completion_request->messages) {
    cJSON *itemLocal = chat_message_convertToJSON(messagesListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(messages, itemLocal);
    }
    }


    // chat_completion_request->model
    if (!chat_completion_request->model) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "model", chat_completion_request->model) == NULL) {
    goto fail; //String
    }


    // chat_completion_request->reasoning_effort
    if(chat_completion_request->reasoning_effort) {
    if(cJSON_AddStringToObject(item, "reasoning_effort", chat_completion_request->reasoning_effort) == NULL) {
    goto fail; //String
    }
    }


    // chat_completion_request->temperature
    if(chat_completion_request->temperature) {
    if(cJSON_AddNumberToObject(item, "temperature", chat_completion_request->temperature) == NULL) {
    goto fail; //Numeric
    }
    }


    // chat_completion_request->top_p
    if(chat_completion_request->top_p) {
    if(cJSON_AddNumberToObject(item, "top_p", chat_completion_request->top_p) == NULL) {
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

chat_completion_request_t *chat_completion_request_parseFromJSON(cJSON *chat_completion_requestJSON){

    chat_completion_request_t *chat_completion_request_local_var = NULL;

    // define the local list for chat_completion_request->messages
    list_t *messagesList = NULL;

    // chat_completion_request->max_tokens
    cJSON *max_tokens = cJSON_GetObjectItemCaseSensitive(chat_completion_requestJSON, "max_tokens");
    if (cJSON_IsNull(max_tokens)) {
        max_tokens = NULL;
    }
    if (max_tokens) { 
    if(!cJSON_IsNumber(max_tokens))
    {
    goto end; //Numeric
    }
    }

    // chat_completion_request->messages
    cJSON *messages = cJSON_GetObjectItemCaseSensitive(chat_completion_requestJSON, "messages");
    if (cJSON_IsNull(messages)) {
        messages = NULL;
    }
    if (!messages) {
        goto end;
    }

    
    cJSON *messages_local_nonprimitive = NULL;
    if(!cJSON_IsArray(messages)){
        goto end; //nonprimitive container
    }

    messagesList = list_createList();

    cJSON_ArrayForEach(messages_local_nonprimitive,messages )
    {
        if(!cJSON_IsObject(messages_local_nonprimitive)){
            goto end;
        }
        chat_message_t *messagesItem = chat_message_parseFromJSON(messages_local_nonprimitive);

        list_addElement(messagesList, messagesItem);
    }

    // chat_completion_request->model
    cJSON *model = cJSON_GetObjectItemCaseSensitive(chat_completion_requestJSON, "model");
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

    // chat_completion_request->reasoning_effort
    cJSON *reasoning_effort = cJSON_GetObjectItemCaseSensitive(chat_completion_requestJSON, "reasoning_effort");
    if (cJSON_IsNull(reasoning_effort)) {
        reasoning_effort = NULL;
    }
    if (reasoning_effort) { 
    if(!cJSON_IsString(reasoning_effort) && !cJSON_IsNull(reasoning_effort))
    {
    goto end; //String
    }
    }

    // chat_completion_request->temperature
    cJSON *temperature = cJSON_GetObjectItemCaseSensitive(chat_completion_requestJSON, "temperature");
    if (cJSON_IsNull(temperature)) {
        temperature = NULL;
    }
    if (temperature) { 
    if(!cJSON_IsNumber(temperature))
    {
    goto end; //Numeric
    }
    }

    // chat_completion_request->top_p
    cJSON *top_p = cJSON_GetObjectItemCaseSensitive(chat_completion_requestJSON, "top_p");
    if (cJSON_IsNull(top_p)) {
        top_p = NULL;
    }
    if (top_p) { 
    if(!cJSON_IsNumber(top_p))
    {
    goto end; //Numeric
    }
    }


    chat_completion_request_local_var = chat_completion_request_create_internal (
        max_tokens ? max_tokens->valuedouble : 0,
        messagesList,
        strdup(model->valuestring),
        reasoning_effort && !cJSON_IsNull(reasoning_effort) ? strdup(reasoning_effort->valuestring) : NULL,
        temperature ? temperature->valuedouble : 0,
        top_p ? top_p->valuedouble : 0
        );

    return chat_completion_request_local_var;
end:
    if (messagesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, messagesList) {
            chat_message_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(messagesList);
        messagesList = NULL;
    }
    return NULL;

}
