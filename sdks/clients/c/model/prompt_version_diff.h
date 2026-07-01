/*
 * prompt_version_diff.h
 *
 * 
 */

#ifndef _prompt_version_diff_H_
#define _prompt_version_diff_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_version_diff_t prompt_version_diff_t;

#include "diff_line.h"



typedef struct prompt_version_diff_t {
    char *from_version_id; // string
    list_t *lines; //nonprimitive container
    char *to_version_id; // string

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_version_diff_t;

__attribute__((deprecated)) prompt_version_diff_t *prompt_version_diff_create(
    char *from_version_id,
    list_t *lines,
    char *to_version_id
);

void prompt_version_diff_free(prompt_version_diff_t *prompt_version_diff);

prompt_version_diff_t *prompt_version_diff_parseFromJSON(cJSON *prompt_version_diffJSON);

cJSON *prompt_version_diff_convertToJSON(prompt_version_diff_t *prompt_version_diff);

#endif /* _prompt_version_diff_H_ */

