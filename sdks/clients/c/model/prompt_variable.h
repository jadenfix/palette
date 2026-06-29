/*
 * prompt_variable.h
 *
 * 
 */

#ifndef _prompt_variable_H_
#define _prompt_variable_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_variable_t prompt_variable_t;




typedef struct prompt_variable_t {
    char *_default; // string
    char *description; // string
    char *name; // string
    int required; //boolean

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_variable_t;

__attribute__((deprecated)) prompt_variable_t *prompt_variable_create(
    char *_default,
    char *description,
    char *name,
    int required
);

void prompt_variable_free(prompt_variable_t *prompt_variable);

prompt_variable_t *prompt_variable_parseFromJSON(cJSON *prompt_variableJSON);

cJSON *prompt_variable_convertToJSON(prompt_variable_t *prompt_variable);

#endif /* _prompt_variable_H_ */

