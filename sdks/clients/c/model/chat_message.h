/*
 * chat_message.h
 *
 * A single chat message in the OpenAI-compatible request.
 */

#ifndef _chat_message_H_
#define _chat_message_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct chat_message_t chat_message_t;




typedef struct chat_message_t {
    char *content; // string
    char *role; // string

    int _library_owned; // Is the library responsible for freeing this object?
} chat_message_t;

__attribute__((deprecated)) chat_message_t *chat_message_create(
    char *content,
    char *role
);

void chat_message_free(chat_message_t *chat_message);

chat_message_t *chat_message_parseFromJSON(cJSON *chat_messageJSON);

cJSON *chat_message_convertToJSON(chat_message_t *chat_message);

#endif /* _chat_message_H_ */

