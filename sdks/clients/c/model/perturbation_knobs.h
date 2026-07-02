/*
 * perturbation_knobs.h
 *
 * Tunable knobs describing how a scenario may be perturbed during replay.
 */

#ifndef _perturbation_knobs_H_
#define _perturbation_knobs_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct perturbation_knobs_t perturbation_knobs_t;




typedef struct perturbation_knobs_t {
    int auth_failure; //boolean
    int contradictory_source; //boolean
    int prompt_injection; //boolean
    int stale_source; //boolean
    int timeout; //boolean
    int tool_schema_mismatch; //boolean

    int _library_owned; // Is the library responsible for freeing this object?
} perturbation_knobs_t;

__attribute__((deprecated)) perturbation_knobs_t *perturbation_knobs_create(
    int auth_failure,
    int contradictory_source,
    int prompt_injection,
    int stale_source,
    int timeout,
    int tool_schema_mismatch
);

void perturbation_knobs_free(perturbation_knobs_t *perturbation_knobs);

perturbation_knobs_t *perturbation_knobs_parseFromJSON(cJSON *perturbation_knobsJSON);

cJSON *perturbation_knobs_convertToJSON(perturbation_knobs_t *perturbation_knobs);

#endif /* _perturbation_knobs_H_ */

