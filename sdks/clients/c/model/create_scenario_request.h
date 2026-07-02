/*
 * create_scenario_request.h
 *
 * 
 */

#ifndef _create_scenario_request_H_
#define _create_scenario_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct create_scenario_request_t create_scenario_request_t;

#include "failure_mode.h"



typedef struct create_scenario_request_t {
    char *exemplar_trace_id; // string
    char *expected_outcome; // string
    beater_api_failure_mode__e failure_mode; //referenced enum
    list_t *source_trace_ids; //primitive container
    char *title; // string

    int _library_owned; // Is the library responsible for freeing this object?
} create_scenario_request_t;

__attribute__((deprecated)) create_scenario_request_t *create_scenario_request_create(
    char *exemplar_trace_id,
    char *expected_outcome,
    beater_api_failure_mode__e failure_mode,
    list_t *source_trace_ids,
    char *title
);

void create_scenario_request_free(create_scenario_request_t *create_scenario_request);

create_scenario_request_t *create_scenario_request_parseFromJSON(cJSON *create_scenario_requestJSON);

cJSON *create_scenario_request_convertToJSON(create_scenario_request_t *create_scenario_request);

#endif /* _create_scenario_request_H_ */

