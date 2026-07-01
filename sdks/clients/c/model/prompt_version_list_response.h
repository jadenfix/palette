/*
 * prompt_version_list_response.h
 *
 * 
 */

#ifndef _prompt_version_list_response_H_
#define _prompt_version_list_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_version_list_response_t prompt_version_list_response_t;

#include "prompt_version.h"



typedef struct prompt_version_list_response_t {
    list_t *versions; //nonprimitive container

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_version_list_response_t;

__attribute__((deprecated)) prompt_version_list_response_t *prompt_version_list_response_create(
    list_t *versions
);

void prompt_version_list_response_free(prompt_version_list_response_t *prompt_version_list_response);

prompt_version_list_response_t *prompt_version_list_response_parseFromJSON(cJSON *prompt_version_list_responseJSON);

cJSON *prompt_version_list_response_convertToJSON(prompt_version_list_response_t *prompt_version_list_response);

#endif /* _prompt_version_list_response_H_ */

