/*
 * prompt_list_response.h
 *
 * 
 */

#ifndef _prompt_list_response_H_
#define _prompt_list_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_list_response_t prompt_list_response_t;

#include "prompt.h"



typedef struct prompt_list_response_t {
    list_t *prompts; //nonprimitive container

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_list_response_t;

__attribute__((deprecated)) prompt_list_response_t *prompt_list_response_create(
    list_t *prompts
);

void prompt_list_response_free(prompt_list_response_t *prompt_list_response);

prompt_list_response_t *prompt_list_response_parseFromJSON(cJSON *prompt_list_responseJSON);

cJSON *prompt_list_response_convertToJSON(prompt_list_response_t *prompt_list_response);

#endif /* _prompt_list_response_H_ */

