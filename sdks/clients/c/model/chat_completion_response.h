/*
 * chat_completion_response.h
 *
 * An OpenAI-compatible chat completion response.
 */

#ifndef _chat_completion_response_H_
#define _chat_completion_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct chat_completion_response_t chat_completion_response_t;

#include "chat_completion_choice.h"
#include "chat_completion_usage.h"



typedef struct chat_completion_response_t {
    list_t *choices; //nonprimitive container
    long created; //numeric
    char *id; // string
    char *model; // string
    char *object; // string
    struct chat_completion_usage_t *usage; //model

    int _library_owned; // Is the library responsible for freeing this object?
} chat_completion_response_t;

__attribute__((deprecated)) chat_completion_response_t *chat_completion_response_create(
    list_t *choices,
    long created,
    char *id,
    char *model,
    char *object,
    chat_completion_usage_t *usage
);

void chat_completion_response_free(chat_completion_response_t *chat_completion_response);

chat_completion_response_t *chat_completion_response_parseFromJSON(cJSON *chat_completion_responseJSON);

cJSON *chat_completion_response_convertToJSON(chat_completion_response_t *chat_completion_response);

#endif /* _chat_completion_response_H_ */

