/*
 * add_prompt_version_request.h
 *
 * Request body for &#x60;addPromptVersion&#x60;: a new immutable template revision.
 */

#ifndef _add_prompt_version_request_H_
#define _add_prompt_version_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct add_prompt_version_request_t add_prompt_version_request_t;

#include "prompt_template.h"



typedef struct add_prompt_version_request_t {
    char *created_by; // string
    char *message; // string
    struct prompt_template_t *_template; //model

    int _library_owned; // Is the library responsible for freeing this object?
} add_prompt_version_request_t;

__attribute__((deprecated)) add_prompt_version_request_t *add_prompt_version_request_create(
    char *created_by,
    char *message,
    prompt_template_t *_template
);

void add_prompt_version_request_free(add_prompt_version_request_t *add_prompt_version_request);

add_prompt_version_request_t *add_prompt_version_request_parseFromJSON(cJSON *add_prompt_version_requestJSON);

cJSON *add_prompt_version_request_convertToJSON(add_prompt_version_request_t *add_prompt_version_request);

#endif /* _add_prompt_version_request_H_ */

