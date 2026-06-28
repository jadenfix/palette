/*
 * gateway_outcome.h
 *
 * The result of a successfully proxied completion.
 */

#ifndef _gateway_outcome_H_
#define _gateway_outcome_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct gateway_outcome_t gateway_outcome_t;

#include "chat_completion_response.h"
#include "model_ref.h"
#include "money.h"
#include "token_counts.h"



typedef struct gateway_outcome_t {
    int attempts; //numeric
    int cached; //boolean
    struct money_t *cost; //model
    struct model_ref_t *model; //model
    char *provider; // string
    char *request_hash; // string
    struct chat_completion_response_t *response; //model
    struct token_counts_t *tokens; //model

    int _library_owned; // Is the library responsible for freeing this object?
} gateway_outcome_t;

__attribute__((deprecated)) gateway_outcome_t *gateway_outcome_create(
    int attempts,
    int cached,
    money_t *cost,
    model_ref_t *model,
    char *provider,
    char *request_hash,
    chat_completion_response_t *response,
    token_counts_t *tokens
);

void gateway_outcome_free(gateway_outcome_t *gateway_outcome);

gateway_outcome_t *gateway_outcome_parseFromJSON(cJSON *gateway_outcomeJSON);

cJSON *gateway_outcome_convertToJSON(gateway_outcome_t *gateway_outcome);

#endif /* _gateway_outcome_H_ */

