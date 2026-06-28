/*
 * chat_completion_request.h
 *
 * An OpenAI-compatible chat completion request. The &#x60;model&#x60; field is a free-form model identifier, making the gateway model-agnostic: any provider/model can be named per request, paired with a [&#x60;ModelRouting&#x60;] credential policy.
 */

#ifndef _chat_completion_request_H_
#define _chat_completion_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct chat_completion_request_t chat_completion_request_t;

#include "chat_message.h"



typedef struct chat_completion_request_t {
    long max_tokens; //numeric
    list_t *messages; //nonprimitive container
    char *model; // string
    char *reasoning_effort; // string
    double temperature; //numeric
    double top_p; //numeric

    int _library_owned; // Is the library responsible for freeing this object?
} chat_completion_request_t;

__attribute__((deprecated)) chat_completion_request_t *chat_completion_request_create(
    long max_tokens,
    list_t *messages,
    char *model,
    char *reasoning_effort,
    double temperature,
    double top_p
);

void chat_completion_request_free(chat_completion_request_t *chat_completion_request);

chat_completion_request_t *chat_completion_request_parseFromJSON(cJSON *chat_completion_requestJSON);

cJSON *chat_completion_request_convertToJSON(chat_completion_request_t *chat_completion_request);

#endif /* _chat_completion_request_H_ */

