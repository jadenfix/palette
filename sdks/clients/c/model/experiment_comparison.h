/*
 * experiment_comparison.h
 *
 * 
 */

#ifndef _experiment_comparison_H_
#define _experiment_comparison_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct experiment_comparison_t experiment_comparison_t;

#include "gate_decision.h"
#include "statistical_test.h"



typedef struct experiment_comparison_t {
    double adjusted_alpha; //numeric
    double baseline_mean; //numeric
    double candidate_mean; //numeric
    double ci_high; //numeric
    double ci_low; //numeric
    beater_api_gate_decision__e decision; //referenced enum
    double delta; //numeric
    double mde; //numeric
    double p_value; //numeric
    int required_n; //numeric
    int sample_size; //numeric
    beater_api_statistical_test__e test; //referenced enum

    int _library_owned; // Is the library responsible for freeing this object?
} experiment_comparison_t;

__attribute__((deprecated)) experiment_comparison_t *experiment_comparison_create(
    double adjusted_alpha,
    double baseline_mean,
    double candidate_mean,
    double ci_high,
    double ci_low,
    beater_api_gate_decision__e decision,
    double delta,
    double mde,
    double p_value,
    int required_n,
    int sample_size,
    beater_api_statistical_test__e test
);

void experiment_comparison_free(experiment_comparison_t *experiment_comparison);

experiment_comparison_t *experiment_comparison_parseFromJSON(cJSON *experiment_comparisonJSON);

cJSON *experiment_comparison_convertToJSON(experiment_comparison_t *experiment_comparison);

#endif /* _experiment_comparison_H_ */

