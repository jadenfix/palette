#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "palette_connect_status_response.h"



static palette_connect_status_response_t *palette_connect_status_response_create_internal(
    int first_eval_run,
    int first_trace_received,
    int ok,
    char *project_id,
    beater_api_palette_connect_status__e status,
    char *tenant_id,
    list_t* totals,
    int usage_configured
    ) {
    palette_connect_status_response_t *palette_connect_status_response_local_var = malloc(sizeof(palette_connect_status_response_t));
    if (!palette_connect_status_response_local_var) {
        return NULL;
    }
    palette_connect_status_response_local_var->first_eval_run = first_eval_run;
    palette_connect_status_response_local_var->first_trace_received = first_trace_received;
    palette_connect_status_response_local_var->ok = ok;
    palette_connect_status_response_local_var->project_id = project_id;
    palette_connect_status_response_local_var->status = status;
    palette_connect_status_response_local_var->tenant_id = tenant_id;
    palette_connect_status_response_local_var->totals = totals;
    palette_connect_status_response_local_var->usage_configured = usage_configured;

    palette_connect_status_response_local_var->_library_owned = 1;
    return palette_connect_status_response_local_var;
}

__attribute__((deprecated)) palette_connect_status_response_t *palette_connect_status_response_create(
    int first_eval_run,
    int first_trace_received,
    int ok,
    char *project_id,
    beater_api_palette_connect_status__e status,
    char *tenant_id,
    list_t* totals,
    int usage_configured
    ) {
    return palette_connect_status_response_create_internal (
        first_eval_run,
        first_trace_received,
        ok,
        project_id,
        status,
        tenant_id,
        totals,
        usage_configured
        );
}

void palette_connect_status_response_free(palette_connect_status_response_t *palette_connect_status_response) {
    if(NULL == palette_connect_status_response){
        return ;
    }
    if(palette_connect_status_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "palette_connect_status_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (palette_connect_status_response->project_id) {
        free(palette_connect_status_response->project_id);
        palette_connect_status_response->project_id = NULL;
    }
    if (palette_connect_status_response->tenant_id) {
        free(palette_connect_status_response->tenant_id);
        palette_connect_status_response->tenant_id = NULL;
    }
    if (palette_connect_status_response->totals) {
        list_ForEach(listEntry, palette_connect_status_response->totals) {
            keyValuePair_t *localKeyValue = listEntry->data;
            free (localKeyValue->key);
            free (localKeyValue->value);
            keyValuePair_free(localKeyValue);
        }
        list_freeList(palette_connect_status_response->totals);
        palette_connect_status_response->totals = NULL;
    }
    free(palette_connect_status_response);
}

cJSON *palette_connect_status_response_convertToJSON(palette_connect_status_response_t *palette_connect_status_response) {
    cJSON *item = cJSON_CreateObject();

    // palette_connect_status_response->first_eval_run
    if (!palette_connect_status_response->first_eval_run) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "first_eval_run", palette_connect_status_response->first_eval_run) == NULL) {
    goto fail; //Bool
    }


    // palette_connect_status_response->first_trace_received
    if (!palette_connect_status_response->first_trace_received) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "first_trace_received", palette_connect_status_response->first_trace_received) == NULL) {
    goto fail; //Bool
    }


    // palette_connect_status_response->ok
    if (!palette_connect_status_response->ok) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "ok", palette_connect_status_response->ok) == NULL) {
    goto fail; //Bool
    }


    // palette_connect_status_response->project_id
    if (!palette_connect_status_response->project_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "project_id", palette_connect_status_response->project_id) == NULL) {
    goto fail; //String
    }


    // palette_connect_status_response->status
    if (beater_api_palette_connect_status__NULL == palette_connect_status_response->status) {
        goto fail;
    }
    cJSON *status_local_JSON = palette_connect_status_convertToJSON(palette_connect_status_response->status);
    if(status_local_JSON == NULL) {
        goto fail; // custom
    }
    cJSON_AddItemToObject(item, "status", status_local_JSON);
    if(item->child == NULL) {
        goto fail;
    }


    // palette_connect_status_response->tenant_id
    if (!palette_connect_status_response->tenant_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "tenant_id", palette_connect_status_response->tenant_id) == NULL) {
    goto fail; //String
    }


    // palette_connect_status_response->totals
    if (!palette_connect_status_response->totals) {
        goto fail;
    }
    cJSON *totals = cJSON_AddObjectToObject(item, "totals");
    if(totals == NULL) {
        goto fail; //primitive map container
    }
    cJSON *localMapObject = totals;
    listEntry_t *totalsListEntry;
    if (palette_connect_status_response->totals) {
    list_ForEach(totalsListEntry, palette_connect_status_response->totals) {
        keyValuePair_t *localKeyValue = totalsListEntry->data;
    }
    }


    // palette_connect_status_response->usage_configured
    if (!palette_connect_status_response->usage_configured) {
        goto fail;
    }
    if(cJSON_AddBoolToObject(item, "usage_configured", palette_connect_status_response->usage_configured) == NULL) {
    goto fail; //Bool
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

palette_connect_status_response_t *palette_connect_status_response_parseFromJSON(cJSON *palette_connect_status_responseJSON){

    palette_connect_status_response_t *palette_connect_status_response_local_var = NULL;

    // define the local variable for palette_connect_status_response->status
    beater_api_palette_connect_status__e status_local_nonprim = 0;

    // define the local map for palette_connect_status_response->totals
    list_t *totalsList = NULL;

    // palette_connect_status_response->first_eval_run
    cJSON *first_eval_run = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "first_eval_run");
    if (cJSON_IsNull(first_eval_run)) {
        first_eval_run = NULL;
    }
    if (!first_eval_run) {
        goto end;
    }

    
    if(!cJSON_IsBool(first_eval_run))
    {
    goto end; //Bool
    }

    // palette_connect_status_response->first_trace_received
    cJSON *first_trace_received = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "first_trace_received");
    if (cJSON_IsNull(first_trace_received)) {
        first_trace_received = NULL;
    }
    if (!first_trace_received) {
        goto end;
    }

    
    if(!cJSON_IsBool(first_trace_received))
    {
    goto end; //Bool
    }

    // palette_connect_status_response->ok
    cJSON *ok = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "ok");
    if (cJSON_IsNull(ok)) {
        ok = NULL;
    }
    if (!ok) {
        goto end;
    }

    
    if(!cJSON_IsBool(ok))
    {
    goto end; //Bool
    }

    // palette_connect_status_response->project_id
    cJSON *project_id = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "project_id");
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

    // palette_connect_status_response->status
    cJSON *status = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "status");
    if (cJSON_IsNull(status)) {
        status = NULL;
    }
    if (!status) {
        goto end;
    }

    
    status_local_nonprim = palette_connect_status_parseFromJSON(status); //custom

    // palette_connect_status_response->tenant_id
    cJSON *tenant_id = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "tenant_id");
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

    // palette_connect_status_response->totals
    cJSON *totals = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "totals");
    if (cJSON_IsNull(totals)) {
        totals = NULL;
    }
    if (!totals) {
        goto end;
    }

    
    cJSON *totals_local_map = NULL;
    if(!cJSON_IsObject(totals) && !cJSON_IsNull(totals))
    {
        goto end;//primitive map container
    }
    if(cJSON_IsObject(totals))
    {
        totalsList = list_createList();
        keyValuePair_t *localMapKeyPair;
        cJSON_ArrayForEach(totals_local_map, totals)
        {
            cJSON *localMapObject = totals_local_map;
            list_addElement(totalsList , localMapKeyPair);
        }
    }

    // palette_connect_status_response->usage_configured
    cJSON *usage_configured = cJSON_GetObjectItemCaseSensitive(palette_connect_status_responseJSON, "usage_configured");
    if (cJSON_IsNull(usage_configured)) {
        usage_configured = NULL;
    }
    if (!usage_configured) {
        goto end;
    }

    
    if(!cJSON_IsBool(usage_configured))
    {
    goto end; //Bool
    }


    palette_connect_status_response_local_var = palette_connect_status_response_create_internal (
        first_eval_run->valueint,
        first_trace_received->valueint,
        ok->valueint,
        strdup(project_id->valuestring),
        status_local_nonprim,
        strdup(tenant_id->valuestring),
        totalsList,
        usage_configured->valueint
        );

    return palette_connect_status_response_local_var;
end:
    if (status_local_nonprim) {
        status_local_nonprim = 0;
    }
    if (totalsList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, totalsList) {
            keyValuePair_t *localKeyValue = listEntry->data;
            free(localKeyValue->key);
            localKeyValue->key = NULL;
            keyValuePair_free(localKeyValue);
            localKeyValue = NULL;
        }
        list_freeList(totalsList);
        totalsList = NULL;
    }
    return NULL;

}
