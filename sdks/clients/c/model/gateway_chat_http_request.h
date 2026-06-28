/*
 * gateway_chat_http_request.h
 *
 * Body for [&#x60;gateway_chat_completions_route&#x60;]: an OpenAI-compatible chat completion request plus the per-request routing policy and an optional budget ceiling. An empty &#x60;routing&#x60; means \&quot;I did not bring a key\&quot; — the gateway falls back to its managed default model if one is configured (hosted), otherwise it returns a typed no-key error (OSS).
 */

#ifndef _gateway_chat_http_request_H_
#define _gateway_chat_http_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct gateway_chat_http_request_t gateway_chat_http_request_t;

#include "chat_completion_request.h"
#include "model_routing.h"



typedef struct gateway_chat_http_request_t {
    long budget_micros; //numeric
    struct chat_completion_request_t *completion; //model
    list_t *routing; //nonprimitive container

    int _library_owned; // Is the library responsible for freeing this object?
} gateway_chat_http_request_t;

__attribute__((deprecated)) gateway_chat_http_request_t *gateway_chat_http_request_create(
    long budget_micros,
    chat_completion_request_t *completion,
    list_t *routing
);

void gateway_chat_http_request_free(gateway_chat_http_request_t *gateway_chat_http_request);

gateway_chat_http_request_t *gateway_chat_http_request_parseFromJSON(cJSON *gateway_chat_http_requestJSON);

cJSON *gateway_chat_http_request_convertToJSON(gateway_chat_http_request_t *gateway_chat_http_request);

#endif /* _gateway_chat_http_request_H_ */

