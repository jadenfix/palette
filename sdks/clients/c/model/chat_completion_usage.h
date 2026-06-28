/*
 * chat_completion_usage.h
 *
 * Token usage in the OpenAI-compatible response shape.
 */

#ifndef _chat_completion_usage_H_
#define _chat_completion_usage_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct chat_completion_usage_t chat_completion_usage_t;




typedef struct chat_completion_usage_t {
    long completion_tokens; //numeric
    long prompt_tokens; //numeric
    long total_tokens; //numeric

    int _library_owned; // Is the library responsible for freeing this object?
} chat_completion_usage_t;

__attribute__((deprecated)) chat_completion_usage_t *chat_completion_usage_create(
    long completion_tokens,
    long prompt_tokens,
    long total_tokens
);

void chat_completion_usage_free(chat_completion_usage_t *chat_completion_usage);

chat_completion_usage_t *chat_completion_usage_parseFromJSON(cJSON *chat_completion_usageJSON);

cJSON *chat_completion_usage_convertToJSON(chat_completion_usage_t *chat_completion_usage);

#endif /* _chat_completion_usage_H_ */

