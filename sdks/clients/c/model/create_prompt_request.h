/*
 * create_prompt_request.h
 *
 * Request body for &#x60;createPrompt&#x60;: the new prompt&#39;s metadata plus its initial (version 1) template.
 */

#ifndef _create_prompt_request_H_
#define _create_prompt_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct create_prompt_request_t create_prompt_request_t;

#include "prompt_template.h"



typedef struct create_prompt_request_t {
    char *created_by; // string
    char *description; // string
    char *message; // string
    char *name; // string
    struct prompt_template_t *_template; //model

    int _library_owned; // Is the library responsible for freeing this object?
} create_prompt_request_t;

__attribute__((deprecated)) create_prompt_request_t *create_prompt_request_create(
    char *created_by,
    char *description,
    char *message,
    char *name,
    prompt_template_t *_template
);

void create_prompt_request_free(create_prompt_request_t *create_prompt_request);

create_prompt_request_t *create_prompt_request_parseFromJSON(cJSON *create_prompt_requestJSON);

cJSON *create_prompt_request_convertToJSON(create_prompt_request_t *create_prompt_request);

#endif /* _create_prompt_request_H_ */

