#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "mine_scenarios_request.h"



static mine_scenarios_request_t *mine_scenarios_request_create_internal(
    double jaccard_threshold,
    list_t *trace_ids
    ) {
    mine_scenarios_request_t *mine_scenarios_request_local_var = malloc(sizeof(mine_scenarios_request_t));
    if (!mine_scenarios_request_local_var) {
        return NULL;
    }
    mine_scenarios_request_local_var->jaccard_threshold = jaccard_threshold;
    mine_scenarios_request_local_var->trace_ids = trace_ids;

    mine_scenarios_request_local_var->_library_owned = 1;
    return mine_scenarios_request_local_var;
}

__attribute__((deprecated)) mine_scenarios_request_t *mine_scenarios_request_create(
    double jaccard_threshold,
    list_t *trace_ids
    ) {
    return mine_scenarios_request_create_internal (
        jaccard_threshold,
        trace_ids
        );
}

void mine_scenarios_request_free(mine_scenarios_request_t *mine_scenarios_request) {
    if(NULL == mine_scenarios_request){
        return ;
    }
    if(mine_scenarios_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "mine_scenarios_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (mine_scenarios_request->trace_ids) {
        list_ForEach(listEntry, mine_scenarios_request->trace_ids) {
            free(listEntry->data);
        }
        list_freeList(mine_scenarios_request->trace_ids);
        mine_scenarios_request->trace_ids = NULL;
    }
    free(mine_scenarios_request);
}

cJSON *mine_scenarios_request_convertToJSON(mine_scenarios_request_t *mine_scenarios_request) {
    cJSON *item = cJSON_CreateObject();

    // mine_scenarios_request->jaccard_threshold
    if(mine_scenarios_request->jaccard_threshold) {
    if(cJSON_AddNumberToObject(item, "jaccard_threshold", mine_scenarios_request->jaccard_threshold) == NULL) {
    goto fail; //Numeric
    }
    }


    // mine_scenarios_request->trace_ids
    if (!mine_scenarios_request->trace_ids) {
        goto fail;
    }
    cJSON *trace_ids = cJSON_AddArrayToObject(item, "trace_ids");
    if(trace_ids == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *trace_idsListEntry;
    list_ForEach(trace_idsListEntry, mine_scenarios_request->trace_ids) {
    if(cJSON_AddStringToObject(trace_ids, "", trace_idsListEntry->data) == NULL)
    {
        goto fail;
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

mine_scenarios_request_t *mine_scenarios_request_parseFromJSON(cJSON *mine_scenarios_requestJSON){

    mine_scenarios_request_t *mine_scenarios_request_local_var = NULL;

    // define the local list for mine_scenarios_request->trace_ids
    list_t *trace_idsList = NULL;

    // mine_scenarios_request->jaccard_threshold
    cJSON *jaccard_threshold = cJSON_GetObjectItemCaseSensitive(mine_scenarios_requestJSON, "jaccard_threshold");
    if (cJSON_IsNull(jaccard_threshold)) {
        jaccard_threshold = NULL;
    }
    if (jaccard_threshold) { 
    if(!cJSON_IsNumber(jaccard_threshold))
    {
    goto end; //Numeric
    }
    }

    // mine_scenarios_request->trace_ids
    cJSON *trace_ids = cJSON_GetObjectItemCaseSensitive(mine_scenarios_requestJSON, "trace_ids");
    if (cJSON_IsNull(trace_ids)) {
        trace_ids = NULL;
    }
    if (!trace_ids) {
        goto end;
    }

    
    cJSON *trace_ids_local = NULL;
    if(!cJSON_IsArray(trace_ids)) {
        goto end;//primitive container
    }
    trace_idsList = list_createList();

    cJSON_ArrayForEach(trace_ids_local, trace_ids)
    {
        if(!cJSON_IsString(trace_ids_local))
        {
            goto end;
        }
        list_addElement(trace_idsList , strdup(trace_ids_local->valuestring));
    }


    mine_scenarios_request_local_var = mine_scenarios_request_create_internal (
        jaccard_threshold ? jaccard_threshold->valuedouble : 0,
        trace_idsList
        );

    return mine_scenarios_request_local_var;
end:
    if (trace_idsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, trace_idsList) {
            free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(trace_idsList);
        trace_idsList = NULL;
    }
    return NULL;

}
