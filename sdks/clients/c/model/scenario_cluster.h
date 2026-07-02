/*
 * scenario_cluster.h
 *
 * A cluster of failing traces that share a similar failure signature.
 */

#ifndef _scenario_cluster_H_
#define _scenario_cluster_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct scenario_cluster_t scenario_cluster_t;

#include "failure_mode.h"
#include "signature.h"



typedef struct scenario_cluster_t {
    beater_api_failure_mode__e dominant_failure_mode; //referenced enum
    char *exemplar_trace_id; // string
    list_t *member_trace_ids; //primitive container
    struct signature_t *signature; //model
    int size; //numeric

    int _library_owned; // Is the library responsible for freeing this object?
} scenario_cluster_t;

__attribute__((deprecated)) scenario_cluster_t *scenario_cluster_create(
    beater_api_failure_mode__e dominant_failure_mode,
    char *exemplar_trace_id,
    list_t *member_trace_ids,
    signature_t *signature,
    int size
);

void scenario_cluster_free(scenario_cluster_t *scenario_cluster);

scenario_cluster_t *scenario_cluster_parseFromJSON(cJSON *scenario_clusterJSON);

cJSON *scenario_cluster_convertToJSON(scenario_cluster_t *scenario_cluster);

#endif /* _scenario_cluster_H_ */

