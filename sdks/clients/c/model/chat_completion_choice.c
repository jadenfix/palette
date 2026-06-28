#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "chat_completion_choice.h"



static chat_completion_choice_t *chat_completion_choice_create_internal(
    char *finish_reason,
    int index,
    chat_message_t *message
    ) {
    chat_completion_choice_t *chat_completion_choice_local_var = malloc(sizeof(chat_completion_choice_t));
    if (!chat_completion_choice_local_var) {
        return NULL;
    }
    chat_completion_choice_local_var->finish_reason = finish_reason;
    chat_completion_choice_local_var->index = index;
    chat_completion_choice_local_var->message = message;

    chat_completion_choice_local_var->_library_owned = 1;
    return chat_completion_choice_local_var;
}

__attribute__((deprecated)) chat_completion_choice_t *chat_completion_choice_create(
    char *finish_reason,
    int index,
    chat_message_t *message
    ) {
    return chat_completion_choice_create_internal (
        finish_reason,
        index,
        message
        );
}

void chat_completion_choice_free(chat_completion_choice_t *chat_completion_choice) {
    if(NULL == chat_completion_choice){
        return ;
    }
    if(chat_completion_choice->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "chat_completion_choice_free");
        return ;
    }
    listEntry_t *listEntry;
    if (chat_completion_choice->finish_reason) {
        free(chat_completion_choice->finish_reason);
        chat_completion_choice->finish_reason = NULL;
    }
    if (chat_completion_choice->message) {
        chat_message_free(chat_completion_choice->message);
        chat_completion_choice->message = NULL;
    }
    free(chat_completion_choice);
}

cJSON *chat_completion_choice_convertToJSON(chat_completion_choice_t *chat_completion_choice) {
    cJSON *item = cJSON_CreateObject();

    // chat_completion_choice->finish_reason
    if(chat_completion_choice->finish_reason) {
    if(cJSON_AddStringToObject(item, "finish_reason", chat_completion_choice->finish_reason) == NULL) {
    goto fail; //String
    }
    }


    // chat_completion_choice->index
    if (!chat_completion_choice->index) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "index", chat_completion_choice->index) == NULL) {
    goto fail; //Numeric
    }


    // chat_completion_choice->message
    if (!chat_completion_choice->message) {
        goto fail;
    }
    cJSON *message_local_JSON = chat_message_convertToJSON(chat_completion_choice->message);
    if(message_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "message", message_local_JSON);
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

chat_completion_choice_t *chat_completion_choice_parseFromJSON(cJSON *chat_completion_choiceJSON){

    chat_completion_choice_t *chat_completion_choice_local_var = NULL;

    // define the local variable for chat_completion_choice->message
    chat_message_t *message_local_nonprim = NULL;

    // chat_completion_choice->finish_reason
    cJSON *finish_reason = cJSON_GetObjectItemCaseSensitive(chat_completion_choiceJSON, "finish_reason");
    if (cJSON_IsNull(finish_reason)) {
        finish_reason = NULL;
    }
    if (finish_reason) { 
    if(!cJSON_IsString(finish_reason) && !cJSON_IsNull(finish_reason))
    {
    goto end; //String
    }
    }

    // chat_completion_choice->index
    cJSON *index = cJSON_GetObjectItemCaseSensitive(chat_completion_choiceJSON, "index");
    if (cJSON_IsNull(index)) {
        index = NULL;
    }
    if (!index) {
        goto end;
    }

    
    if(!cJSON_IsNumber(index))
    {
    goto end; //Numeric
    }

    // chat_completion_choice->message
    cJSON *message = cJSON_GetObjectItemCaseSensitive(chat_completion_choiceJSON, "message");
    if (cJSON_IsNull(message)) {
        message = NULL;
    }
    if (!message) {
        goto end;
    }

    
    message_local_nonprim = chat_message_parseFromJSON(message); //nonprimitive


    chat_completion_choice_local_var = chat_completion_choice_create_internal (
        finish_reason && !cJSON_IsNull(finish_reason) ? strdup(finish_reason->valuestring) : NULL,
        index->valuedouble,
        message_local_nonprim
        );

    return chat_completion_choice_local_var;
end:
    if (message_local_nonprim) {
        chat_message_free(message_local_nonprim);
        message_local_nonprim = NULL;
    }
    return NULL;

}
