/*
 * prompt.h
 *
 * 
 */

#ifndef _prompt_H_
#define _prompt_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_t prompt_t;




typedef struct prompt_t {
    char *created_at; //date time
    char *description; // string
    char *name; // string
    char *project_id; // string
    char *prompt_id; // string
    char *tenant_id; // string
    char *updated_at; //date time

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_t;

__attribute__((deprecated)) prompt_t *prompt_create(
    char *created_at,
    char *description,
    char *name,
    char *project_id,
    char *prompt_id,
    char *tenant_id,
    char *updated_at
);

void prompt_free(prompt_t *prompt);

prompt_t *prompt_parseFromJSON(cJSON *promptJSON);

cJSON *prompt_convertToJSON(prompt_t *prompt);

#endif /* _prompt_H_ */

