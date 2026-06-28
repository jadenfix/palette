#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "experiment_comparison.h"



static experiment_comparison_t *experiment_comparison_create_internal(
    double adjusted_alpha,
    double baseline_mean,
    double candidate_mean,
    double ci_high,
    double ci_low,
    beater_api_gate_decision__e decision,
    double delta,
    double p_value,
    int sample_size,
    beater_api_statistical_test__e test
    ) {
    experiment_comparison_t *experiment_comparison_local_var = malloc(sizeof(experiment_comparison_t));
    if (!experiment_comparison_local_var) {
        return NULL;
    }
    experiment_comparison_local_var->adjusted_alpha = adjusted_alpha;
    experiment_comparison_local_var->baseline_mean = baseline_mean;
    experiment_comparison_local_var->candidate_mean = candidate_mean;
    experiment_comparison_local_var->ci_high = ci_high;
    experiment_comparison_local_var->ci_low = ci_low;
    experiment_comparison_local_var->decision = decision;
    experiment_comparison_local_var->delta = delta;
    experiment_comparison_local_var->p_value = p_value;
    experiment_comparison_local_var->sample_size = sample_size;
    experiment_comparison_local_var->test = test;

    experiment_comparison_local_var->_library_owned = 1;
    return experiment_comparison_local_var;
}

__attribute__((deprecated)) experiment_comparison_t *experiment_comparison_create(
    double adjusted_alpha,
    double baseline_mean,
    double candidate_mean,
    double ci_high,
    double ci_low,
    beater_api_gate_decision__e decision,
    double delta,
    double p_value,
    int sample_size,
    beater_api_statistical_test__e test
    ) {
    return experiment_comparison_create_internal (
        adjusted_alpha,
        baseline_mean,
        candidate_mean,
        ci_high,
        ci_low,
        decision,
        delta,
        p_value,
        sample_size,
        test
        );
}

void experiment_comparison_free(experiment_comparison_t *experiment_comparison) {
    if(NULL == experiment_comparison){
        return ;
    }
    if(experiment_comparison->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "experiment_comparison_free");
        return ;
    }
    listEntry_t *listEntry;
    free(experiment_comparison);
}

cJSON *experiment_comparison_convertToJSON(experiment_comparison_t *experiment_comparison) {
    cJSON *item = cJSON_CreateObject();

    // experiment_comparison->adjusted_alpha
    if (!experiment_comparison->adjusted_alpha) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "adjusted_alpha", experiment_comparison->adjusted_alpha) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->baseline_mean
    if (!experiment_comparison->baseline_mean) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "baseline_mean", experiment_comparison->baseline_mean) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->candidate_mean
    if (!experiment_comparison->candidate_mean) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "candidate_mean", experiment_comparison->candidate_mean) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->ci_high
    if (!experiment_comparison->ci_high) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "ci_high", experiment_comparison->ci_high) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->ci_low
    if (!experiment_comparison->ci_low) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "ci_low", experiment_comparison->ci_low) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->decision
    if (beater_api_gate_decision__NULL == experiment_comparison->decision) {
        goto fail;
    }
    cJSON *decision_local_JSON = gate_decision_convertToJSON(experiment_comparison->decision);
    if(decision_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "decision", decision_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }


    // experiment_comparison->delta
    if (!experiment_comparison->delta) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "delta", experiment_comparison->delta) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->p_value
    if (!experiment_comparison->p_value) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "p_value", experiment_comparison->p_value) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->sample_size
    if (!experiment_comparison->sample_size) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "sample_size", experiment_comparison->sample_size) == NULL) {
    goto fail; //Numeric
    }


    // experiment_comparison->test
    if (beater_api_statistical_test__NULL == experiment_comparison->test) {
        goto fail;
    }
    cJSON *test_local_JSON = statistical_test_convertToJSON(experiment_comparison->test);
    if(test_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "test", test_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

experiment_comparison_t *experiment_comparison_parseFromJSON(cJSON *experiment_comparisonJSON){

    experiment_comparison_t *experiment_comparison_local_var = NULL;

    // define the local variable for experiment_comparison->decision
    beater_api_gate_decision__e decision_local_nonprim = 0;

    // define the local variable for experiment_comparison->test
    beater_api_statistical_test__e test_local_nonprim = 0;

    // experiment_comparison->adjusted_alpha
    cJSON *adjusted_alpha = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "adjusted_alpha");
    if (cJSON_IsNull(adjusted_alpha)) {
        adjusted_alpha = NULL;
    }
    if (!adjusted_alpha) {
        goto end;
    }

    
    if(!cJSON_IsNumber(adjusted_alpha))
    {
    goto end; //Numeric
    }

    // experiment_comparison->baseline_mean
    cJSON *baseline_mean = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "baseline_mean");
    if (cJSON_IsNull(baseline_mean)) {
        baseline_mean = NULL;
    }
    if (!baseline_mean) {
        goto end;
    }

    
    if(!cJSON_IsNumber(baseline_mean))
    {
    goto end; //Numeric
    }

    // experiment_comparison->candidate_mean
    cJSON *candidate_mean = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "candidate_mean");
    if (cJSON_IsNull(candidate_mean)) {
        candidate_mean = NULL;
    }
    if (!candidate_mean) {
        goto end;
    }

    
    if(!cJSON_IsNumber(candidate_mean))
    {
    goto end; //Numeric
    }

    // experiment_comparison->ci_high
    cJSON *ci_high = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "ci_high");
    if (cJSON_IsNull(ci_high)) {
        ci_high = NULL;
    }
    if (!ci_high) {
        goto end;
    }

    
    if(!cJSON_IsNumber(ci_high))
    {
    goto end; //Numeric
    }

    // experiment_comparison->ci_low
    cJSON *ci_low = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "ci_low");
    if (cJSON_IsNull(ci_low)) {
        ci_low = NULL;
    }
    if (!ci_low) {
        goto end;
    }

    
    if(!cJSON_IsNumber(ci_low))
    {
    goto end; //Numeric
    }

    // experiment_comparison->decision
    cJSON *decision = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "decision");
    if (cJSON_IsNull(decision)) {
        decision = NULL;
    }
    if (!decision) {
        goto end;
    }

    
    decision_local_nonprim = gate_decision_parseFromJSON(decision); //custom

    // experiment_comparison->delta
    cJSON *delta = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "delta");
    if (cJSON_IsNull(delta)) {
        delta = NULL;
    }
    if (!delta) {
        goto end;
    }

    
    if(!cJSON_IsNumber(delta))
    {
    goto end; //Numeric
    }

    // experiment_comparison->p_value
    cJSON *p_value = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "p_value");
    if (cJSON_IsNull(p_value)) {
        p_value = NULL;
    }
    if (!p_value) {
        goto end;
    }

    
    if(!cJSON_IsNumber(p_value))
    {
    goto end; //Numeric
    }

    // experiment_comparison->sample_size
    cJSON *sample_size = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "sample_size");
    if (cJSON_IsNull(sample_size)) {
        sample_size = NULL;
    }
    if (!sample_size) {
        goto end;
    }

    
    if(!cJSON_IsNumber(sample_size))
    {
    goto end; //Numeric
    }

    // experiment_comparison->test
    cJSON *test = cJSON_GetObjectItemCaseSensitive(experiment_comparisonJSON, "test");
    if (cJSON_IsNull(test)) {
        test = NULL;
    }
    if (!test) {
        goto end;
    }

    
    test_local_nonprim = statistical_test_parseFromJSON(test); //custom


    experiment_comparison_local_var = experiment_comparison_create_internal (
        adjusted_alpha->valuedouble,
        baseline_mean->valuedouble,
        candidate_mean->valuedouble,
        ci_high->valuedouble,
        ci_low->valuedouble,
        decision_local_nonprim,
        delta->valuedouble,
        p_value->valuedouble,
        sample_size->valuedouble,
        test_local_nonprim
        );

    return experiment_comparison_local_var;
end:
    if (decision_local_nonprim) {
        decision_local_nonprim = 0;
    }
    if (test_local_nonprim) {
        test_local_nonprim = 0;
    }
    return NULL;

}
