#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "chat_message.h"



static chat_message_t *chat_message_create_internal(
    char *content,
    char *role
    ) {
    chat_message_t *chat_message_local_var = malloc(sizeof(chat_message_t));
    if (!chat_message_local_var) {
        return NULL;
    }
    chat_message_local_var->content = content;
    chat_message_local_var->role = role;

    chat_message_local_var->_library_owned = 1;
    return chat_message_local_var;
}

__attribute__((deprecated)) chat_message_t *chat_message_create(
    char *content,
    char *role
    ) {
    return chat_message_create_internal (
        content,
        role
        );
}

void chat_message_free(chat_message_t *chat_message) {
    if(NULL == chat_message){
        return ;
    }
    if(chat_message->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "chat_message_free");
        return ;
    }
    listEntry_t *listEntry;
    if (chat_message->content) {
        free(chat_message->content);
        chat_message->content = NULL;
    }
    if (chat_message->role) {
        free(chat_message->role);
        chat_message->role = NULL;
    }
    free(chat_message);
}

cJSON *chat_message_convertToJSON(chat_message_t *chat_message) {
    cJSON *item = cJSON_CreateObject();

    // chat_message->content
    if (!chat_message->content) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "content", chat_message->content) == NULL) {
    goto fail; //String
    }


    // chat_message->role
    if (!chat_message->role) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "role", chat_message->role) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

chat_message_t *chat_message_parseFromJSON(cJSON *chat_messageJSON){

    chat_message_t *chat_message_local_var = NULL;

    // chat_message->content
    cJSON *content = cJSON_GetObjectItemCaseSensitive(chat_messageJSON, "content");
    if (cJSON_IsNull(content)) {
        content = NULL;
    }
    if (!content) {
        goto end;
    }

    
    if(!cJSON_IsString(content))
    {
    goto end; //String
    }

    // chat_message->role
    cJSON *role = cJSON_GetObjectItemCaseSensitive(chat_messageJSON, "role");
    if (cJSON_IsNull(role)) {
        role = NULL;
    }
    if (!role) {
        goto end;
    }

    
    if(!cJSON_IsString(role))
    {
    goto end; //String
    }


    chat_message_local_var = chat_message_create_internal (
        strdup(content->valuestring),
        strdup(role->valuestring)
        );

    return chat_message_local_var;
end:
    return NULL;

}
