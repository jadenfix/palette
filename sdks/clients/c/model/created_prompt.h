/*
 * created_prompt.h
 *
 * 
 */

#ifndef _created_prompt_H_
#define _created_prompt_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct created_prompt_t created_prompt_t;

#include "prompt.h"
#include "prompt_version.h"



typedef struct created_prompt_t {
    struct prompt_t *prompt; //model
    struct prompt_version_t *version; //model

    int _library_owned; // Is the library responsible for freeing this object?
} created_prompt_t;

__attribute__((deprecated)) created_prompt_t *created_prompt_create(
    prompt_t *prompt,
    prompt_version_t *version
);

void created_prompt_free(created_prompt_t *created_prompt);

created_prompt_t *created_prompt_parseFromJSON(cJSON *created_promptJSON);

cJSON *created_prompt_convertToJSON(created_prompt_t *created_prompt);

#endif /* _created_prompt_H_ */

