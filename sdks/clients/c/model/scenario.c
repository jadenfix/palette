#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "scenario.h"



static scenario_t *scenario_create_internal(
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
    ) {
    scenario_t *scenario_local_var = malloc(sizeof(scenario_t));
    if (!scenario_local_var) {
        return NULL;
    }
    scenario_local_var->created_at = created_at;
    scenario_local_var->exemplar_trace_id = exemplar_trace_id;
    scenario_local_var->expected_outcome = expected_outcome;
    scenario_local_var->failure_mode = failure_mode;
    scenario_local_var->perturbation_knobs = perturbation_knobs;
    scenario_local_var->recurrence_count = recurrence_count;
    scenario_local_var->redaction_class = redaction_class;
    scenario_local_var->scenario_id = scenario_id;
    scenario_local_var->scope = scope;
    scenario_local_var->source_trace_ids = source_trace_ids;
    scenario_local_var->title = title;

    scenario_local_var->_library_owned = 1;
    return scenario_local_var;
}

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
    ) {
    return scenario_create_internal (
        created_at,
        exemplar_trace_id,
        expected_outcome,
        failure_mode,
        perturbation_knobs,
        recurrence_count,
        redaction_class,
        scenario_id,
        scope,
        source_trace_ids,
        title
        );
}

void scenario_free(scenario_t *scenario) {
    if(NULL == scenario){
        return ;
    }
    if(scenario->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "scenario_free");
        return ;
    }
    listEntry_t *listEntry;
    if (scenario->created_at) {
        free(scenario->created_at);
        scenario->created_at = NULL;
    }
    if (scenario->exemplar_trace_id) {
        free(scenario->exemplar_trace_id);
        scenario->exemplar_trace_id = NULL;
    }
    if (scenario->expected_outcome) {
        free(scenario->expected_outcome);
        scenario->expected_outcome = NULL;
    }
    if (scenario->perturbation_knobs) {
        perturbation_knobs_free(scenario->perturbation_knobs);
        scenario->perturbation_knobs = NULL;
    }
    if (scenario->scenario_id) {
        free(scenario->scenario_id);
        scenario->scenario_id = NULL;
    }
    if (scenario->scope) {
        tenant_scope_free(scenario->scope);
        scenario->scope = NULL;
    }
    if (scenario->source_trace_ids) {
        list_ForEach(listEntry, scenario->source_trace_ids) {
            free(listEntry->data);
        }
        list_freeList(scenario->source_trace_ids);
        scenario->source_trace_ids = NULL;
    }
    if (scenario->title) {
        free(scenario->title);
        scenario->title = NULL;
    }
    free(scenario);
}

cJSON *scenario_convertToJSON(scenario_t *scenario) {
    cJSON *item = cJSON_CreateObject();

    // scenario->created_at
    if (!scenario->created_at) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "created_at", scenario->created_at) == NULL) {
    goto fail; //Date-Time
    }


    // scenario->exemplar_trace_id
    if (!scenario->exemplar_trace_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "exemplar_trace_id", scenario->exemplar_trace_id) == NULL) {
    goto fail; //String
    }


    // scenario->expected_outcome
    if(scenario->expected_outcome) {
    if(cJSON_AddStringToObject(item, "expected_outcome", scenario->expected_outcome) == NULL) {
    goto fail; //String
    }
    }


    // scenario->failure_mode
    if (beater_api_failure_mode__NULL == scenario->failure_mode) {
        goto fail;
    }
    cJSON *failure_mode_local_JSON = failure_mode_convertToJSON(scenario->failure_mode);
    if(failure_mode_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "failure_mode", failure_mode_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }


    // scenario->perturbation_knobs
    if (!scenario->perturbation_knobs) {
        goto fail;
    }
    cJSON *perturbation_knobs_local_JSON = perturbation_knobs_convertToJSON(scenario->perturbation_knobs);
    if(perturbation_knobs_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "perturbation_knobs", perturbation_knobs_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // scenario->recurrence_count
    if (!scenario->recurrence_count) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "recurrence_count", scenario->recurrence_count) == NULL) {
    goto fail; //Numeric
    }


    // scenario->redaction_class
    if (beater_api_redaction_class__NULL == scenario->redaction_class) {
        goto fail;
    }
    cJSON *redaction_class_local_JSON = redaction_class_convertToJSON(scenario->redaction_class);
    if(redaction_class_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "redaction_class", redaction_class_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }


    // scenario->scenario_id
    if (!scenario->scenario_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "scenario_id", scenario->scenario_id) == NULL) {
    goto fail; //String
    }


    // scenario->scope
    if (!scenario->scope) {
        goto fail;
    }
    cJSON *scope_local_JSON = tenant_scope_convertToJSON(scenario->scope);
    if(scope_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "scope", scope_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // scenario->source_trace_ids
    if (!scenario->source_trace_ids) {
        goto fail;
    }
    cJSON *source_trace_ids = cJSON_AddArrayToObject(item, "source_trace_ids");
    if(source_trace_ids == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *source_trace_idsListEntry;
    list_ForEach(source_trace_idsListEntry, scenario->source_trace_ids) {
    if(cJSON_AddStringToObject(source_trace_ids, "", source_trace_idsListEntry->data) == NULL)
    {
        goto fail;
    }
    }


    // scenario->title
    if (!scenario->title) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "title", scenario->title) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

scenario_t *scenario_parseFromJSON(cJSON *scenarioJSON){

    scenario_t *scenario_local_var = NULL;

    // define the local variable for scenario->failure_mode
    beater_api_failure_mode__e failure_mode_local_nonprim = 0;

    // define the local variable for scenario->perturbation_knobs
    perturbation_knobs_t *perturbation_knobs_local_nonprim = NULL;

    // define the local variable for scenario->redaction_class
    beater_api_redaction_class__e redaction_class_local_nonprim = 0;

    // define the local variable for scenario->scope
    tenant_scope_t *scope_local_nonprim = NULL;

    // define the local list for scenario->source_trace_ids
    list_t *source_trace_idsList = NULL;

    // scenario->created_at
    cJSON *created_at = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "created_at");
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

    // scenario->exemplar_trace_id
    cJSON *exemplar_trace_id = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "exemplar_trace_id");
    if (cJSON_IsNull(exemplar_trace_id)) {
        exemplar_trace_id = NULL;
    }
    if (!exemplar_trace_id) {
        goto end;
    }

    
    if(!cJSON_IsString(exemplar_trace_id))
    {
    goto end; //String
    }

    // scenario->expected_outcome
    cJSON *expected_outcome = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "expected_outcome");
    if (cJSON_IsNull(expected_outcome)) {
        expected_outcome = NULL;
    }
    if (expected_outcome) { 
    if(!cJSON_IsString(expected_outcome) && !cJSON_IsNull(expected_outcome))
    {
    goto end; //String
    }
    }

    // scenario->failure_mode
    cJSON *failure_mode = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "failure_mode");
    if (cJSON_IsNull(failure_mode)) {
        failure_mode = NULL;
    }
    if (!failure_mode) {
        goto end;
    }

    
    failure_mode_local_nonprim = failure_mode_parseFromJSON(failure_mode); //custom

    // scenario->perturbation_knobs
    cJSON *perturbation_knobs = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "perturbation_knobs");
    if (cJSON_IsNull(perturbation_knobs)) {
        perturbation_knobs = NULL;
    }
    if (!perturbation_knobs) {
        goto end;
    }

    
    perturbation_knobs_local_nonprim = perturbation_knobs_parseFromJSON(perturbation_knobs); //nonprimitive

    // scenario->recurrence_count
    cJSON *recurrence_count = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "recurrence_count");
    if (cJSON_IsNull(recurrence_count)) {
        recurrence_count = NULL;
    }
    if (!recurrence_count) {
        goto end;
    }

    
    if(!cJSON_IsNumber(recurrence_count))
    {
    goto end; //Numeric
    }

    // scenario->redaction_class
    cJSON *redaction_class = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "redaction_class");
    if (cJSON_IsNull(redaction_class)) {
        redaction_class = NULL;
    }
    if (!redaction_class) {
        goto end;
    }

    
    redaction_class_local_nonprim = redaction_class_parseFromJSON(redaction_class); //custom

    // scenario->scenario_id
    cJSON *scenario_id = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "scenario_id");
    if (cJSON_IsNull(scenario_id)) {
        scenario_id = NULL;
    }
    if (!scenario_id) {
        goto end;
    }

    
    if(!cJSON_IsString(scenario_id))
    {
    goto end; //String
    }

    // scenario->scope
    cJSON *scope = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "scope");
    if (cJSON_IsNull(scope)) {
        scope = NULL;
    }
    if (!scope) {
        goto end;
    }

    
    scope_local_nonprim = tenant_scope_parseFromJSON(scope); //nonprimitive

    // scenario->source_trace_ids
    cJSON *source_trace_ids = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "source_trace_ids");
    if (cJSON_IsNull(source_trace_ids)) {
        source_trace_ids = NULL;
    }
    if (!source_trace_ids) {
        goto end;
    }

    
    cJSON *source_trace_ids_local = NULL;
    if(!cJSON_IsArray(source_trace_ids)) {
        goto end;//primitive container
    }
    source_trace_idsList = list_createList();

    cJSON_ArrayForEach(source_trace_ids_local, source_trace_ids)
    {
        if(!cJSON_IsString(source_trace_ids_local))
        {
            goto end;
        }
        list_addElement(source_trace_idsList , strdup(source_trace_ids_local->valuestring));
    }

    // scenario->title
    cJSON *title = cJSON_GetObjectItemCaseSensitive(scenarioJSON, "title");
    if (cJSON_IsNull(title)) {
        title = NULL;
    }
    if (!title) {
        goto end;
    }

    
    if(!cJSON_IsString(title))
    {
    goto end; //String
    }


    scenario_local_var = scenario_create_internal (
        strdup(created_at->valuestring),
        strdup(exemplar_trace_id->valuestring),
        expected_outcome && !cJSON_IsNull(expected_outcome) ? strdup(expected_outcome->valuestring) : NULL,
        failure_mode_local_nonprim,
        perturbation_knobs_local_nonprim,
        recurrence_count->valuedouble,
        redaction_class_local_nonprim,
        strdup(scenario_id->valuestring),
        scope_local_nonprim,
        source_trace_idsList,
        strdup(title->valuestring)
        );

    return scenario_local_var;
end:
    if (failure_mode_local_nonprim) {
        failure_mode_local_nonprim = 0;
    }
    if (perturbation_knobs_local_nonprim) {
        perturbation_knobs_free(perturbation_knobs_local_nonprim);
        perturbation_knobs_local_nonprim = NULL;
    }
    if (redaction_class_local_nonprim) {
        redaction_class_local_nonprim = 0;
    }
    if (scope_local_nonprim) {
        tenant_scope_free(scope_local_nonprim);
        scope_local_nonprim = NULL;
    }
    if (source_trace_idsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, source_trace_idsList) {
            free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(source_trace_idsList);
        source_trace_idsList = NULL;
    }
    return NULL;

}
