#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "calibration_report.h"



static calibration_report_t *calibration_report_create_internal(
    double brier_score,
    char *calibration_report_id,
    double cohen_kappa,
    double cohen_kappa_ci_high,
    double cohen_kappa_ci_low,
    calibration_confusion_t *confusion,
    char *created_at,
    char *dataset_id,
    char *dataset_version_id,
    char *eval_report_id,
    char *evaluator_version_id,
    double expected_agreement,
    double expected_calibration_error,
    list_t *items,
    double observed_agreement,
    double observed_agreement_ci_high,
    double observed_agreement_ci_low,
    calibration_policy_t *policy,
    char *project_id,
    list_t *reliability_bins,
    int sample_count,
    char *tenant_id
    ) {
    calibration_report_t *calibration_report_local_var = malloc(sizeof(calibration_report_t));
    if (!calibration_report_local_var) {
        return NULL;
    }
    calibration_report_local_var->brier_score = brier_score;
    calibration_report_local_var->calibration_report_id = calibration_report_id;
    calibration_report_local_var->cohen_kappa = cohen_kappa;
    calibration_report_local_var->cohen_kappa_ci_high = cohen_kappa_ci_high;
    calibration_report_local_var->cohen_kappa_ci_low = cohen_kappa_ci_low;
    calibration_report_local_var->confusion = confusion;
    calibration_report_local_var->created_at = created_at;
    calibration_report_local_var->dataset_id = dataset_id;
    calibration_report_local_var->dataset_version_id = dataset_version_id;
    calibration_report_local_var->eval_report_id = eval_report_id;
    calibration_report_local_var->evaluator_version_id = evaluator_version_id;
    calibration_report_local_var->expected_agreement = expected_agreement;
    calibration_report_local_var->expected_calibration_error = expected_calibration_error;
    calibration_report_local_var->items = items;
    calibration_report_local_var->observed_agreement = observed_agreement;
    calibration_report_local_var->observed_agreement_ci_high = observed_agreement_ci_high;
    calibration_report_local_var->observed_agreement_ci_low = observed_agreement_ci_low;
    calibration_report_local_var->policy = policy;
    calibration_report_local_var->project_id = project_id;
    calibration_report_local_var->reliability_bins = reliability_bins;
    calibration_report_local_var->sample_count = sample_count;
    calibration_report_local_var->tenant_id = tenant_id;

    calibration_report_local_var->_library_owned = 1;
    return calibration_report_local_var;
}

__attribute__((deprecated)) calibration_report_t *calibration_report_create(
    double brier_score,
    char *calibration_report_id,
    double cohen_kappa,
    double cohen_kappa_ci_high,
    double cohen_kappa_ci_low,
    calibration_confusion_t *confusion,
    char *created_at,
    char *dataset_id,
    char *dataset_version_id,
    char *eval_report_id,
    char *evaluator_version_id,
    double expected_agreement,
    double expected_calibration_error,
    list_t *items,
    double observed_agreement,
    double observed_agreement_ci_high,
    double observed_agreement_ci_low,
    calibration_policy_t *policy,
    char *project_id,
    list_t *reliability_bins,
    int sample_count,
    char *tenant_id
    ) {
    return calibration_report_create_internal (
        brier_score,
        calibration_report_id,
        cohen_kappa,
        cohen_kappa_ci_high,
        cohen_kappa_ci_low,
        confusion,
        created_at,
        dataset_id,
        dataset_version_id,
        eval_report_id,
        evaluator_version_id,
        expected_agreement,
        expected_calibration_error,
        items,
        observed_agreement,
        observed_agreement_ci_high,
        observed_agreement_ci_low,
        policy,
        project_id,
        reliability_bins,
        sample_count,
        tenant_id
        );
}

void calibration_report_free(calibration_report_t *calibration_report) {
    if(NULL == calibration_report){
        return ;
    }
    if(calibration_report->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "calibration_report_free");
        return ;
    }
    listEntry_t *listEntry;
    if (calibration_report->calibration_report_id) {
        free(calibration_report->calibration_report_id);
        calibration_report->calibration_report_id = NULL;
    }
    if (calibration_report->confusion) {
        calibration_confusion_free(calibration_report->confusion);
        calibration_report->confusion = NULL;
    }
    if (calibration_report->created_at) {
        free(calibration_report->created_at);
        calibration_report->created_at = NULL;
    }
    if (calibration_report->dataset_id) {
        free(calibration_report->dataset_id);
        calibration_report->dataset_id = NULL;
    }
    if (calibration_report->dataset_version_id) {
        free(calibration_report->dataset_version_id);
        calibration_report->dataset_version_id = NULL;
    }
    if (calibration_report->eval_report_id) {
        free(calibration_report->eval_report_id);
        calibration_report->eval_report_id = NULL;
    }
    if (calibration_report->evaluator_version_id) {
        free(calibration_report->evaluator_version_id);
        calibration_report->evaluator_version_id = NULL;
    }
    if (calibration_report->items) {
        list_ForEach(listEntry, calibration_report->items) {
            calibration_item_free(listEntry->data);
        }
        list_freeList(calibration_report->items);
        calibration_report->items = NULL;
    }
    if (calibration_report->policy) {
        calibration_policy_free(calibration_report->policy);
        calibration_report->policy = NULL;
    }
    if (calibration_report->project_id) {
        free(calibration_report->project_id);
        calibration_report->project_id = NULL;
    }
    if (calibration_report->reliability_bins) {
        list_ForEach(listEntry, calibration_report->reliability_bins) {
            reliability_bin_free(listEntry->data);
        }
        list_freeList(calibration_report->reliability_bins);
        calibration_report->reliability_bins = NULL;
    }
    if (calibration_report->tenant_id) {
        free(calibration_report->tenant_id);
        calibration_report->tenant_id = NULL;
    }
    free(calibration_report);
}

cJSON *calibration_report_convertToJSON(calibration_report_t *calibration_report) {
    cJSON *item = cJSON_CreateObject();

    // calibration_report->brier_score
    if (!calibration_report->brier_score) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "brier_score", calibration_report->brier_score) == NULL) {
    goto fail; //Numeric
    }


    // calibration_report->calibration_report_id
    if (!calibration_report->calibration_report_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "calibration_report_id", calibration_report->calibration_report_id) == NULL) {
    goto fail; //String
    }


    // calibration_report->cohen_kappa
    if (!calibration_report->cohen_kappa) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "cohen_kappa", calibration_report->cohen_kappa) == NULL) {
    goto fail; //Numeric
    }


    // calibration_report->cohen_kappa_ci_high
    if(calibration_report->cohen_kappa_ci_high) {
    if(cJSON_AddNumberToObject(item, "cohen_kappa_ci_high", calibration_report->cohen_kappa_ci_high) == NULL) {
    goto fail; //Numeric
    }
    }


    // calibration_report->cohen_kappa_ci_low
    if(calibration_report->cohen_kappa_ci_low) {
    if(cJSON_AddNumberToObject(item, "cohen_kappa_ci_low", calibration_report->cohen_kappa_ci_low) == NULL) {
    goto fail; //Numeric
    }
    }


    // calibration_report->confusion
    if (!calibration_report->confusion) {
        goto fail;
    }
    cJSON *confusion_local_JSON = calibration_confusion_convertToJSON(calibration_report->confusion);
    if(confusion_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "confusion", confusion_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // calibration_report->created_at
    if (!calibration_report->created_at) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "created_at", calibration_report->created_at) == NULL) {
    goto fail; //Date-Time
    }


    // calibration_report->dataset_id
    if (!calibration_report->dataset_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "dataset_id", calibration_report->dataset_id) == NULL) {
    goto fail; //String
    }


    // calibration_report->dataset_version_id
    if (!calibration_report->dataset_version_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "dataset_version_id", calibration_report->dataset_version_id) == NULL) {
    goto fail; //String
    }


    // calibration_report->eval_report_id
    if (!calibration_report->eval_report_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "eval_report_id", calibration_report->eval_report_id) == NULL) {
    goto fail; //String
    }


    // calibration_report->evaluator_version_id
    if (!calibration_report->evaluator_version_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "evaluator_version_id", calibration_report->evaluator_version_id) == NULL) {
    goto fail; //String
    }


    // calibration_report->expected_agreement
    if (!calibration_report->expected_agreement) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "expected_agreement", calibration_report->expected_agreement) == NULL) {
    goto fail; //Numeric
    }


    // calibration_report->expected_calibration_error
    if (!calibration_report->expected_calibration_error) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "expected_calibration_error", calibration_report->expected_calibration_error) == NULL) {
    goto fail; //Numeric
    }


    // calibration_report->items
    if (!calibration_report->items) {
        goto fail;
    }
    cJSON *items = cJSON_AddArrayToObject(item, "items");
    if(items == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *itemsListEntry;
    if (calibration_report->items) {
    list_ForEach(itemsListEntry, calibration_report->items) {
    cJSON *itemLocal = calibration_item_convertToJSON(itemsListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(items, itemLocal);
    }
    }


    // calibration_report->observed_agreement
    if (!calibration_report->observed_agreement) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "observed_agreement", calibration_report->observed_agreement) == NULL) {
    goto fail; //Numeric
    }


    // calibration_report->observed_agreement_ci_high
    if(calibration_report->observed_agreement_ci_high) {
    if(cJSON_AddNumberToObject(item, "observed_agreement_ci_high", calibration_report->observed_agreement_ci_high) == NULL) {
    goto fail; //Numeric
    }
    }


    // calibration_report->observed_agreement_ci_low
    if(calibration_report->observed_agreement_ci_low) {
    if(cJSON_AddNumberToObject(item, "observed_agreement_ci_low", calibration_report->observed_agreement_ci_low) == NULL) {
    goto fail; //Numeric
    }
    }


    // calibration_report->policy
    if (!calibration_report->policy) {
        goto fail;
    }
    cJSON *policy_local_JSON = calibration_policy_convertToJSON(calibration_report->policy);
    if(policy_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "policy", policy_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // calibration_report->project_id
    if (!calibration_report->project_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "project_id", calibration_report->project_id) == NULL) {
    goto fail; //String
    }


    // calibration_report->reliability_bins
    if (!calibration_report->reliability_bins) {
        goto fail;
    }
    cJSON *reliability_bins = cJSON_AddArrayToObject(item, "reliability_bins");
    if(reliability_bins == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *reliability_binsListEntry;
    if (calibration_report->reliability_bins) {
    list_ForEach(reliability_binsListEntry, calibration_report->reliability_bins) {
    cJSON *itemLocal = reliability_bin_convertToJSON(reliability_binsListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(reliability_bins, itemLocal);
    }
    }


    // calibration_report->sample_count
    if (!calibration_report->sample_count) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "sample_count", calibration_report->sample_count) == NULL) {
    goto fail; //Numeric
    }


    // calibration_report->tenant_id
    if (!calibration_report->tenant_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "tenant_id", calibration_report->tenant_id) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

calibration_report_t *calibration_report_parseFromJSON(cJSON *calibration_reportJSON){

    calibration_report_t *calibration_report_local_var = NULL;

    // define the local variable for calibration_report->confusion
    calibration_confusion_t *confusion_local_nonprim = NULL;

    // define the local list for calibration_report->items
    list_t *itemsList = NULL;

    // define the local variable for calibration_report->policy
    calibration_policy_t *policy_local_nonprim = NULL;

    // define the local list for calibration_report->reliability_bins
    list_t *reliability_binsList = NULL;

    // calibration_report->brier_score
    cJSON *brier_score = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "brier_score");
    if (cJSON_IsNull(brier_score)) {
        brier_score = NULL;
    }
    if (!brier_score) {
        goto end;
    }

    
    if(!cJSON_IsNumber(brier_score))
    {
    goto end; //Numeric
    }

    // calibration_report->calibration_report_id
    cJSON *calibration_report_id = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "calibration_report_id");
    if (cJSON_IsNull(calibration_report_id)) {
        calibration_report_id = NULL;
    }
    if (!calibration_report_id) {
        goto end;
    }

    
    if(!cJSON_IsString(calibration_report_id))
    {
    goto end; //String
    }

    // calibration_report->cohen_kappa
    cJSON *cohen_kappa = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "cohen_kappa");
    if (cJSON_IsNull(cohen_kappa)) {
        cohen_kappa = NULL;
    }
    if (!cohen_kappa) {
        goto end;
    }

    
    if(!cJSON_IsNumber(cohen_kappa))
    {
    goto end; //Numeric
    }

    // calibration_report->cohen_kappa_ci_high
    cJSON *cohen_kappa_ci_high = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "cohen_kappa_ci_high");
    if (cJSON_IsNull(cohen_kappa_ci_high)) {
        cohen_kappa_ci_high = NULL;
    }
    if (cohen_kappa_ci_high) { 
    if(!cJSON_IsNumber(cohen_kappa_ci_high))
    {
    goto end; //Numeric
    }
    }

    // calibration_report->cohen_kappa_ci_low
    cJSON *cohen_kappa_ci_low = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "cohen_kappa_ci_low");
    if (cJSON_IsNull(cohen_kappa_ci_low)) {
        cohen_kappa_ci_low = NULL;
    }
    if (cohen_kappa_ci_low) { 
    if(!cJSON_IsNumber(cohen_kappa_ci_low))
    {
    goto end; //Numeric
    }
    }

    // calibration_report->confusion
    cJSON *confusion = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "confusion");
    if (cJSON_IsNull(confusion)) {
        confusion = NULL;
    }
    if (!confusion) {
        goto end;
    }

    
    confusion_local_nonprim = calibration_confusion_parseFromJSON(confusion); //nonprimitive

    // calibration_report->created_at
    cJSON *created_at = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "created_at");
    if (cJSON_IsNull(created_at)) {
        created_at = NULL;
    }
    if (!created_at) {
        goto end;
    }

    
    if(!cJSON_IsString(created_at) && !cJSON_IsNull(created_at))
    {
    goto end; //DateTime
    }

    // calibration_report->dataset_id
    cJSON *dataset_id = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "dataset_id");
    if (cJSON_IsNull(dataset_id)) {
        dataset_id = NULL;
    }
    if (!dataset_id) {
        goto end;
    }

    
    if(!cJSON_IsString(dataset_id))
    {
    goto end; //String
    }

    // calibration_report->dataset_version_id
    cJSON *dataset_version_id = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "dataset_version_id");
    if (cJSON_IsNull(dataset_version_id)) {
        dataset_version_id = NULL;
    }
    if (!dataset_version_id) {
        goto end;
    }

    
    if(!cJSON_IsString(dataset_version_id))
    {
    goto end; //String
    }

    // calibration_report->eval_report_id
    cJSON *eval_report_id = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "eval_report_id");
    if (cJSON_IsNull(eval_report_id)) {
        eval_report_id = NULL;
    }
    if (!eval_report_id) {
        goto end;
    }

    
    if(!cJSON_IsString(eval_report_id))
    {
    goto end; //String
    }

    // calibration_report->evaluator_version_id
    cJSON *evaluator_version_id = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "evaluator_version_id");
    if (cJSON_IsNull(evaluator_version_id)) {
        evaluator_version_id = NULL;
    }
    if (!evaluator_version_id) {
        goto end;
    }

    
    if(!cJSON_IsString(evaluator_version_id))
    {
    goto end; //String
    }

    // calibration_report->expected_agreement
    cJSON *expected_agreement = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "expected_agreement");
    if (cJSON_IsNull(expected_agreement)) {
        expected_agreement = NULL;
    }
    if (!expected_agreement) {
        goto end;
    }

    
    if(!cJSON_IsNumber(expected_agreement))
    {
    goto end; //Numeric
    }

    // calibration_report->expected_calibration_error
    cJSON *expected_calibration_error = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "expected_calibration_error");
    if (cJSON_IsNull(expected_calibration_error)) {
        expected_calibration_error = NULL;
    }
    if (!expected_calibration_error) {
        goto end;
    }

    
    if(!cJSON_IsNumber(expected_calibration_error))
    {
    goto end; //Numeric
    }

    // calibration_report->items
    cJSON *items = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "items");
    if (cJSON_IsNull(items)) {
        items = NULL;
    }
    if (!items) {
        goto end;
    }

    
    cJSON *items_local_nonprimitive = NULL;
    if(!cJSON_IsArray(items)){
        goto end; //nonprimitive container
    }

    itemsList = list_createList();

    cJSON_ArrayForEach(items_local_nonprimitive,items )
    {
        if(!cJSON_IsObject(items_local_nonprimitive)){
            goto end;
        }
        calibration_item_t *itemsItem = calibration_item_parseFromJSON(items_local_nonprimitive);

        list_addElement(itemsList, itemsItem);
    }

    // calibration_report->observed_agreement
    cJSON *observed_agreement = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "observed_agreement");
    if (cJSON_IsNull(observed_agreement)) {
        observed_agreement = NULL;
    }
    if (!observed_agreement) {
        goto end;
    }

    
    if(!cJSON_IsNumber(observed_agreement))
    {
    goto end; //Numeric
    }

    // calibration_report->observed_agreement_ci_high
    cJSON *observed_agreement_ci_high = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "observed_agreement_ci_high");
    if (cJSON_IsNull(observed_agreement_ci_high)) {
        observed_agreement_ci_high = NULL;
    }
    if (observed_agreement_ci_high) { 
    if(!cJSON_IsNumber(observed_agreement_ci_high))
    {
    goto end; //Numeric
    }
    }

    // calibration_report->observed_agreement_ci_low
    cJSON *observed_agreement_ci_low = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "observed_agreement_ci_low");
    if (cJSON_IsNull(observed_agreement_ci_low)) {
        observed_agreement_ci_low = NULL;
    }
    if (observed_agreement_ci_low) { 
    if(!cJSON_IsNumber(observed_agreement_ci_low))
    {
    goto end; //Numeric
    }
    }

    // calibration_report->policy
    cJSON *policy = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "policy");
    if (cJSON_IsNull(policy)) {
        policy = NULL;
    }
    if (!policy) {
        goto end;
    }

    
    policy_local_nonprim = calibration_policy_parseFromJSON(policy); //nonprimitive

    // calibration_report->project_id
    cJSON *project_id = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "project_id");
    if (cJSON_IsNull(project_id)) {
        project_id = NULL;
    }
    if (!project_id) {
        goto end;
    }

    
    if(!cJSON_IsString(project_id))
    {
    goto end; //String
    }

    // calibration_report->reliability_bins
    cJSON *reliability_bins = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "reliability_bins");
    if (cJSON_IsNull(reliability_bins)) {
        reliability_bins = NULL;
    }
    if (!reliability_bins) {
        goto end;
    }

    
    cJSON *reliability_bins_local_nonprimitive = NULL;
    if(!cJSON_IsArray(reliability_bins)){
        goto end; //nonprimitive container
    }

    reliability_binsList = list_createList();

    cJSON_ArrayForEach(reliability_bins_local_nonprimitive,reliability_bins )
    {
        if(!cJSON_IsObject(reliability_bins_local_nonprimitive)){
            goto end;
        }
        reliability_bin_t *reliability_binsItem = reliability_bin_parseFromJSON(reliability_bins_local_nonprimitive);

        list_addElement(reliability_binsList, reliability_binsItem);
    }

    // calibration_report->sample_count
    cJSON *sample_count = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "sample_count");
    if (cJSON_IsNull(sample_count)) {
        sample_count = NULL;
    }
    if (!sample_count) {
        goto end;
    }

    
    if(!cJSON_IsNumber(sample_count))
    {
    goto end; //Numeric
    }

    // calibration_report->tenant_id
    cJSON *tenant_id = cJSON_GetObjectItemCaseSensitive(calibration_reportJSON, "tenant_id");
    if (cJSON_IsNull(tenant_id)) {
        tenant_id = NULL;
    }
    if (!tenant_id) {
        goto end;
    }

    
    if(!cJSON_IsString(tenant_id))
    {
    goto end; //String
    }


    calibration_report_local_var = calibration_report_create_internal (
        brier_score->valuedouble,
        strdup(calibration_report_id->valuestring),
        cohen_kappa->valuedouble,
        cohen_kappa_ci_high ? cohen_kappa_ci_high->valuedouble : 0,
        cohen_kappa_ci_low ? cohen_kappa_ci_low->valuedouble : 0,
        confusion_local_nonprim,
        strdup(created_at->valuestring),
        strdup(dataset_id->valuestring),
        strdup(dataset_version_id->valuestring),
        strdup(eval_report_id->valuestring),
        strdup(evaluator_version_id->valuestring),
        expected_agreement->valuedouble,
        expected_calibration_error->valuedouble,
        itemsList,
        observed_agreement->valuedouble,
        observed_agreement_ci_high ? observed_agreement_ci_high->valuedouble : 0,
        observed_agreement_ci_low ? observed_agreement_ci_low->valuedouble : 0,
        policy_local_nonprim,
        strdup(project_id->valuestring),
        reliability_binsList,
        sample_count->valuedouble,
        strdup(tenant_id->valuestring)
        );

    return calibration_report_local_var;
end:
    if (confusion_local_nonprim) {
        calibration_confusion_free(confusion_local_nonprim);
        confusion_local_nonprim = NULL;
    }
    if (itemsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, itemsList) {
            calibration_item_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(itemsList);
        itemsList = NULL;
    }
    if (policy_local_nonprim) {
        calibration_policy_free(policy_local_nonprim);
        policy_local_nonprim = NULL;
    }
    if (reliability_binsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, reliability_binsList) {
            reliability_bin_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(reliability_binsList);
        reliability_binsList = NULL;
    }
    return NULL;

}
