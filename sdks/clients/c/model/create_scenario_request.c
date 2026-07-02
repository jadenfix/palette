#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "create_scenario_request.h"



static create_scenario_request_t *create_scenario_request_create_internal(
    char *exemplar_trace_id,
    char *expected_outcome,
    beater_api_failure_mode__e failure_mode,
    list_t *source_trace_ids,
    char *title
    ) {
    create_scenario_request_t *create_scenario_request_local_var = malloc(sizeof(create_scenario_request_t));
    if (!create_scenario_request_local_var) {
        return NULL;
    }
    create_scenario_request_local_var->exemplar_trace_id = exemplar_trace_id;
    create_scenario_request_local_var->expected_outcome = expected_outcome;
    create_scenario_request_local_var->failure_mode = failure_mode;
    create_scenario_request_local_var->source_trace_ids = source_trace_ids;
    create_scenario_request_local_var->title = title;

    create_scenario_request_local_var->_library_owned = 1;
    return create_scenario_request_local_var;
}

__attribute__((deprecated)) create_scenario_request_t *create_scenario_request_create(
    char *exemplar_trace_id,
    char *expected_outcome,
    beater_api_failure_mode__e failure_mode,
    list_t *source_trace_ids,
    char *title
    ) {
    return create_scenario_request_create_internal (
        exemplar_trace_id,
        expected_outcome,
        failure_mode,
        source_trace_ids,
        title
        );
}

void create_scenario_request_free(create_scenario_request_t *create_scenario_request) {
    if(NULL == create_scenario_request){
        return ;
    }
    if(create_scenario_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "create_scenario_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (create_scenario_request->exemplar_trace_id) {
        free(create_scenario_request->exemplar_trace_id);
        create_scenario_request->exemplar_trace_id = NULL;
    }
    if (create_scenario_request->expected_outcome) {
        free(create_scenario_request->expected_outcome);
        create_scenario_request->expected_outcome = NULL;
    }
    if (create_scenario_request->source_trace_ids) {
        list_ForEach(listEntry, create_scenario_request->source_trace_ids) {
            free(listEntry->data);
        }
        list_freeList(create_scenario_request->source_trace_ids);
        create_scenario_request->source_trace_ids = NULL;
    }
    if (create_scenario_request->title) {
        free(create_scenario_request->title);
        create_scenario_request->title = NULL;
    }
    free(create_scenario_request);
}

cJSON *create_scenario_request_convertToJSON(create_scenario_request_t *create_scenario_request) {
    cJSON *item = cJSON_CreateObject();

    // create_scenario_request->exemplar_trace_id
    if(create_scenario_request->exemplar_trace_id) {
    if(cJSON_AddStringToObject(item, "exemplar_trace_id", create_scenario_request->exemplar_trace_id) == NULL) {
    goto fail; //String
    }
    }


    // create_scenario_request->expected_outcome
    if(create_scenario_request->expected_outcome) {
    if(cJSON_AddStringToObject(item, "expected_outcome", create_scenario_request->expected_outcome) == NULL) {
    goto fail; //String
    }
    }


    // create_scenario_request->failure_mode
    if(create_scenario_request->failure_mode != beater_api_failure_mode__NULL) {
    cJSON *failure_mode_local_JSON = failure_mode_convertToJSON(create_scenario_request->failure_mode);
    if(failure_mode_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "failure_mode", failure_mode_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }
    }


    // create_scenario_request->source_trace_ids
    if (!create_scenario_request->source_trace_ids) {
        goto fail;
    }
    cJSON *source_trace_ids = cJSON_AddArrayToObject(item, "source_trace_ids");
    if(source_trace_ids == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *source_trace_idsListEntry;
    list_ForEach(source_trace_idsListEntry, create_scenario_request->source_trace_ids) {
    if(cJSON_AddStringToObject(source_trace_ids, "", source_trace_idsListEntry->data) == NULL)
    {
        goto fail;
    }
    }


    // create_scenario_request->title
    if (!create_scenario_request->title) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "title", create_scenario_request->title) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

create_scenario_request_t *create_scenario_request_parseFromJSON(cJSON *create_scenario_requestJSON){

    create_scenario_request_t *create_scenario_request_local_var = NULL;

    // define the local variable for create_scenario_request->failure_mode
    beater_api_failure_mode__e failure_mode_local_nonprim = 0;

    // define the local list for create_scenario_request->source_trace_ids
    list_t *source_trace_idsList = NULL;

    // create_scenario_request->exemplar_trace_id
    cJSON *exemplar_trace_id = cJSON_GetObjectItemCaseSensitive(create_scenario_requestJSON, "exemplar_trace_id");
    if (cJSON_IsNull(exemplar_trace_id)) {
        exemplar_trace_id = NULL;
    }
    if (exemplar_trace_id) { 
    if(!cJSON_IsString(exemplar_trace_id) && !cJSON_IsNull(exemplar_trace_id))
    {
    goto end; //String
    }
    }

    // create_scenario_request->expected_outcome
    cJSON *expected_outcome = cJSON_GetObjectItemCaseSensitive(create_scenario_requestJSON, "expected_outcome");
    if (cJSON_IsNull(expected_outcome)) {
        expected_outcome = NULL;
    }
    if (expected_outcome) { 
    if(!cJSON_IsString(expected_outcome) && !cJSON_IsNull(expected_outcome))
    {
    goto end; //String
    }
    }

    // create_scenario_request->failure_mode
    cJSON *failure_mode = cJSON_GetObjectItemCaseSensitive(create_scenario_requestJSON, "failure_mode");
    if (cJSON_IsNull(failure_mode)) {
        failure_mode = NULL;
    }
    if (failure_mode) { 
    failure_mode_local_nonprim = failure_mode_parseFromJSON(failure_mode); //custom
    }

    // create_scenario_request->source_trace_ids
    cJSON *source_trace_ids = cJSON_GetObjectItemCaseSensitive(create_scenario_requestJSON, "source_trace_ids");
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

    // create_scenario_request->title
    cJSON *title = cJSON_GetObjectItemCaseSensitive(create_scenario_requestJSON, "title");
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


    create_scenario_request_local_var = create_scenario_request_create_internal (
        exemplar_trace_id && !cJSON_IsNull(exemplar_trace_id) ? strdup(exemplar_trace_id->valuestring) : NULL,
        expected_outcome && !cJSON_IsNull(expected_outcome) ? strdup(expected_outcome->valuestring) : NULL,
        failure_mode ? failure_mode_local_nonprim : 0,
        source_trace_idsList,
        strdup(title->valuestring)
        );

    return create_scenario_request_local_var;
end:
    if (failure_mode_local_nonprim) {
        failure_mode_local_nonprim = 0;
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
