/*
 * mine_scenarios_response.h
 *
 * 
 */

#ifndef _mine_scenarios_response_H_
#define _mine_scenarios_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct mine_scenarios_response_t mine_scenarios_response_t;

#include "scenario_cluster.h"



typedef struct mine_scenarios_response_t {
    list_t *clusters; //nonprimitive container

    int _library_owned; // Is the library responsible for freeing this object?
} mine_scenarios_response_t;

__attribute__((deprecated)) mine_scenarios_response_t *mine_scenarios_response_create(
    list_t *clusters
);

void mine_scenarios_response_free(mine_scenarios_response_t *mine_scenarios_response);

mine_scenarios_response_t *mine_scenarios_response_parseFromJSON(cJSON *mine_scenarios_responseJSON);

cJSON *mine_scenarios_response_convertToJSON(mine_scenarios_response_t *mine_scenarios_response);

#endif /* _mine_scenarios_response_H_ */

