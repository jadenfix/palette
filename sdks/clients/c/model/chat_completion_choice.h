/*
 * chat_completion_choice.h
 *
 * One choice in an OpenAI-compatible response.
 */

#ifndef _chat_completion_choice_H_
#define _chat_completion_choice_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct chat_completion_choice_t chat_completion_choice_t;

#include "chat_message.h"



typedef struct chat_completion_choice_t {
    char *finish_reason; // string
    int index; //numeric
    struct chat_message_t *message; //model

    int _library_owned; // Is the library responsible for freeing this object?
} chat_completion_choice_t;

__attribute__((deprecated)) chat_completion_choice_t *chat_completion_choice_create(
    char *finish_reason,
    int index,
    chat_message_t *message
);

void chat_completion_choice_free(chat_completion_choice_t *chat_completion_choice);

chat_completion_choice_t *chat_completion_choice_parseFromJSON(cJSON *chat_completion_choiceJSON);

cJSON *chat_completion_choice_convertToJSON(chat_completion_choice_t *chat_completion_choice);

#endif /* _chat_completion_choice_H_ */

