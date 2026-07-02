/*
 * list_scenarios_response.h
 *
 * 
 */

#ifndef _list_scenarios_response_H_
#define _list_scenarios_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct list_scenarios_response_t list_scenarios_response_t;

#include "scenario.h"



typedef struct list_scenarios_response_t {
    char *next_cursor; // string
    list_t *scenarios; //nonprimitive container

    int _library_owned; // Is the library responsible for freeing this object?
} list_scenarios_response_t;

__attribute__((deprecated)) list_scenarios_response_t *list_scenarios_response_create(
    char *next_cursor,
    list_t *scenarios
);

void list_scenarios_response_free(list_scenarios_response_t *list_scenarios_response);

list_scenarios_response_t *list_scenarios_response_parseFromJSON(cJSON *list_scenarios_responseJSON);

cJSON *list_scenarios_response_convertToJSON(list_scenarios_response_t *list_scenarios_response);

#endif /* _list_scenarios_response_H_ */

