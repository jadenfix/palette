/*
 * scenario.h
 *
 * A reusable failure scenario mined from production traces.
 */

#ifndef _scenario_H_
#define _scenario_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct scenario_t scenario_t;

#include "failure_mode.h"
#include "perturbation_knobs.h"
#include "redaction_class.h"
#include "tenant_scope.h"



typedef struct scenario_t {
    char *created_at; //date time
    char *exemplar_trace_id; // string
    char *expected_outcome; // string
    beater_api_failure_mode__e failure_mode; //referenced enum
    struct perturbation_knobs_t *perturbation_knobs; //model
    int recurrence_count; //numeric
    beater_api_redaction_class__e redaction_class; //referenced enum
    char *scenario_id; // string
    struct tenant_scope_t *scope; //model
    list_t *source_trace_ids; //primitive container
    char *title; // string

    int _library_owned; // Is the library responsible for freeing this object?
} scenario_t;

__attribute__((deprecated)) scenario_t *scenario_create(
    char *created_at,
    char *exemplar_trace_id,
    char *expected_outcome,
    beater_api_failure_mode__e failure_mode,
    perturbation_knobs_t *perturbation_knobs,
    int recurrence_count,
    beater_api_redaction_class__e redaction_class,
    char *scenario_id,
    tenant_scope_t *scope,
    list_t *source_trace_ids,
    char *title
);

void scenario_free(scenario_t *scenario);

scenario_t *scenario_parseFromJSON(cJSON *scenarioJSON);

cJSON *scenario_convertToJSON(scenario_t *scenario);

#endif /* _scenario_H_ */

