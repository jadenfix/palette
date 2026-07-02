/*
 * mine_scenarios_request.h
 *
 * 
 */

#ifndef _mine_scenarios_request_H_
#define _mine_scenarios_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct mine_scenarios_request_t mine_scenarios_request_t;




typedef struct mine_scenarios_request_t {
    double jaccard_threshold; //numeric
    list_t *trace_ids; //primitive container

    int _library_owned; // Is the library responsible for freeing this object?
} mine_scenarios_request_t;

__attribute__((deprecated)) mine_scenarios_request_t *mine_scenarios_request_create(
    double jaccard_threshold,
    list_t *trace_ids
);

void mine_scenarios_request_free(mine_scenarios_request_t *mine_scenarios_request);

mine_scenarios_request_t *mine_scenarios_request_parseFromJSON(cJSON *mine_scenarios_requestJSON);

cJSON *mine_scenarios_request_convertToJSON(mine_scenarios_request_t *mine_scenarios_request);

#endif /* _mine_scenarios_request_H_ */

