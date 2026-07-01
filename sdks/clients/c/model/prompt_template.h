/*
 * prompt_template.h
 *
 * 
 */

#ifndef _prompt_template_H_
#define _prompt_template_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_template_t prompt_template_t;

#include "prompt_variable.h"



typedef struct prompt_template_t {
    char *body; // string
    list_t *tags; //primitive container
    list_t *variables; //nonprimitive container

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_template_t;

__attribute__((deprecated)) prompt_template_t *prompt_template_create(
    char *body,
    list_t *tags,
    list_t *variables
);

void prompt_template_free(prompt_template_t *prompt_template);

prompt_template_t *prompt_template_parseFromJSON(cJSON *prompt_templateJSON);

cJSON *prompt_template_convertToJSON(prompt_template_t *prompt_template);

#endif /* _prompt_template_H_ */

