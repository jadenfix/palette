#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "scenario_cluster.h"



static scenario_cluster_t *scenario_cluster_create_internal(
    beater_api_failure_mode__e dominant_failure_mode,
    char *exemplar_trace_id,
    list_t *member_trace_ids,
    signature_t *signature,
    int size
    ) {
    scenario_cluster_t *scenario_cluster_local_var = malloc(sizeof(scenario_cluster_t));
    if (!scenario_cluster_local_var) {
        return NULL;
    }
    scenario_cluster_local_var->dominant_failure_mode = dominant_failure_mode;
    scenario_cluster_local_var->exemplar_trace_id = exemplar_trace_id;
    scenario_cluster_local_var->member_trace_ids = member_trace_ids;
    scenario_cluster_local_var->signature = signature;
    scenario_cluster_local_var->size = size;

    scenario_cluster_local_var->_library_owned = 1;
    return scenario_cluster_local_var;
}

__attribute__((deprecated)) scenario_cluster_t *scenario_cluster_create(
    beater_api_failure_mode__e dominant_failure_mode,
    char *exemplar_trace_id,
    list_t *member_trace_ids,
    signature_t *signature,
    int size
    ) {
    return scenario_cluster_create_internal (
        dominant_failure_mode,
        exemplar_trace_id,
        member_trace_ids,
        signature,
        size
        );
}

void scenario_cluster_free(scenario_cluster_t *scenario_cluster) {
    if(NULL == scenario_cluster){
        return ;
    }
    if(scenario_cluster->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "scenario_cluster_free");
        return ;
    }
    listEntry_t *listEntry;
    if (scenario_cluster->exemplar_trace_id) {
        free(scenario_cluster->exemplar_trace_id);
        scenario_cluster->exemplar_trace_id = NULL;
    }
    if (scenario_cluster->member_trace_ids) {
        list_ForEach(listEntry, scenario_cluster->member_trace_ids) {
            free(listEntry->data);
        }
        list_freeList(scenario_cluster->member_trace_ids);
        scenario_cluster->member_trace_ids = NULL;
    }
    if (scenario_cluster->signature) {
        signature_free(scenario_cluster->signature);
        scenario_cluster->signature = NULL;
    }
    free(scenario_cluster);
}

cJSON *scenario_cluster_convertToJSON(scenario_cluster_t *scenario_cluster) {
    cJSON *item = cJSON_CreateObject();

    // scenario_cluster->dominant_failure_mode
    if (beater_api_failure_mode__NULL == scenario_cluster->dominant_failure_mode) {
        goto fail;
    }
    cJSON *dominant_failure_mode_local_JSON = failure_mode_convertToJSON(scenario_cluster->dominant_failure_mode);
    if(dominant_failure_mode_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "dominant_failure_mode", dominant_failure_mode_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }


    // scenario_cluster->exemplar_trace_id
    if (!scenario_cluster->exemplar_trace_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "exemplar_trace_id", scenario_cluster->exemplar_trace_id) == NULL) {
    goto fail; //String
    }


    // scenario_cluster->member_trace_ids
    if (!scenario_cluster->member_trace_ids) {
        goto fail;
    }
    cJSON *member_trace_ids = cJSON_AddArrayToObject(item, "member_trace_ids");
    if(member_trace_ids == NULL) {
        goto fail; //primitive container
    }

    listEntry_t *member_trace_idsListEntry;
    list_ForEach(member_trace_idsListEntry, scenario_cluster->member_trace_ids) {
    if(cJSON_AddStringToObject(member_trace_ids, "", member_trace_idsListEntry->data) == NULL)
    {
        goto fail;
    }
    }


    // scenario_cluster->signature
    if (!scenario_cluster->signature) {
        goto fail;
    }
    cJSON *signature_local_JSON = signature_convertToJSON(scenario_cluster->signature);
    if(signature_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "signature", signature_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // scenario_cluster->size
    if (!scenario_cluster->size) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "size", scenario_cluster->size) == NULL) {
    goto fail; //Numeric
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

scenario_cluster_t *scenario_cluster_parseFromJSON(cJSON *scenario_clusterJSON){

    scenario_cluster_t *scenario_cluster_local_var = NULL;

    // define the local variable for scenario_cluster->dominant_failure_mode
    beater_api_failure_mode__e dominant_failure_mode_local_nonprim = 0;

    // define the local list for scenario_cluster->member_trace_ids
    list_t *member_trace_idsList = NULL;

    // define the local variable for scenario_cluster->signature
    signature_t *signature_local_nonprim = NULL;

    // scenario_cluster->dominant_failure_mode
    cJSON *dominant_failure_mode = cJSON_GetObjectItemCaseSensitive(scenario_clusterJSON, "dominant_failure_mode");
    if (cJSON_IsNull(dominant_failure_mode)) {
        dominant_failure_mode = NULL;
    }
    if (!dominant_failure_mode) {
        goto end;
    }

    
    dominant_failure_mode_local_nonprim = failure_mode_parseFromJSON(dominant_failure_mode); //custom

    // scenario_cluster->exemplar_trace_id
    cJSON *exemplar_trace_id = cJSON_GetObjectItemCaseSensitive(scenario_clusterJSON, "exemplar_trace_id");
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

    // scenario_cluster->member_trace_ids
    cJSON *member_trace_ids = cJSON_GetObjectItemCaseSensitive(scenario_clusterJSON, "member_trace_ids");
    if (cJSON_IsNull(member_trace_ids)) {
        member_trace_ids = NULL;
    }
    if (!member_trace_ids) {
        goto end;
    }

    
    cJSON *member_trace_ids_local = NULL;
    if(!cJSON_IsArray(member_trace_ids)) {
        goto end;//primitive container
    }
    member_trace_idsList = list_createList();

    cJSON_ArrayForEach(member_trace_ids_local, member_trace_ids)
    {
        if(!cJSON_IsString(member_trace_ids_local))
        {
            goto end;
        }
        list_addElement(member_trace_idsList , strdup(member_trace_ids_local->valuestring));
    }

    // scenario_cluster->signature
    cJSON *signature = cJSON_GetObjectItemCaseSensitive(scenario_clusterJSON, "signature");
    if (cJSON_IsNull(signature)) {
        signature = NULL;
    }
    if (!signature) {
        goto end;
    }

    
    signature_local_nonprim = signature_parseFromJSON(signature); //nonprimitive

    // scenario_cluster->size
    cJSON *size = cJSON_GetObjectItemCaseSensitive(scenario_clusterJSON, "size");
    if (cJSON_IsNull(size)) {
        size = NULL;
    }
    if (!size) {
        goto end;
    }

    
    if(!cJSON_IsNumber(size))
    {
    goto end; //Numeric
    }


    scenario_cluster_local_var = scenario_cluster_create_internal (
        dominant_failure_mode_local_nonprim,
        strdup(exemplar_trace_id->valuestring),
        member_trace_idsList,
        signature_local_nonprim,
        size->valuedouble
        );

    return scenario_cluster_local_var;
end:
    if (dominant_failure_mode_local_nonprim) {
        dominant_failure_mode_local_nonprim = 0;
    }
    if (member_trace_idsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, member_trace_idsList) {
            free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(member_trace_idsList);
        member_trace_idsList = NULL;
    }
    if (signature_local_nonprim) {
        signature_free(signature_local_nonprim);
        signature_local_nonprim = NULL;
    }
    return NULL;

}
