#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "dataset_version_snapshot.h"



static dataset_version_snapshot_t *dataset_version_snapshot_create_internal(
    list_t *cases,
    char *corpus_root,
    char *created_at,
    char *dataset_id,
    char *project_id,
    char *tenant_id,
    char *version_id
    ) {
    dataset_version_snapshot_t *dataset_version_snapshot_local_var = malloc(sizeof(dataset_version_snapshot_t));
    if (!dataset_version_snapshot_local_var) {
        return NULL;
    }
    dataset_version_snapshot_local_var->cases = cases;
    dataset_version_snapshot_local_var->corpus_root = corpus_root;
    dataset_version_snapshot_local_var->created_at = created_at;
    dataset_version_snapshot_local_var->dataset_id = dataset_id;
    dataset_version_snapshot_local_var->project_id = project_id;
    dataset_version_snapshot_local_var->tenant_id = tenant_id;
    dataset_version_snapshot_local_var->version_id = version_id;

    dataset_version_snapshot_local_var->_library_owned = 1;
    return dataset_version_snapshot_local_var;
}

__attribute__((deprecated)) dataset_version_snapshot_t *dataset_version_snapshot_create(
    list_t *cases,
    char *corpus_root,
    char *created_at,
    char *dataset_id,
    char *project_id,
    char *tenant_id,
    char *version_id
    ) {
    return dataset_version_snapshot_create_internal (
        cases,
        corpus_root,
        created_at,
        dataset_id,
        project_id,
        tenant_id,
        version_id
        );
}

void dataset_version_snapshot_free(dataset_version_snapshot_t *dataset_version_snapshot) {
    if(NULL == dataset_version_snapshot){
        return ;
    }
    if(dataset_version_snapshot->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "dataset_version_snapshot_free");
        return ;
    }
    listEntry_t *listEntry;
    if (dataset_version_snapshot->cases) {
        list_ForEach(listEntry, dataset_version_snapshot->cases) {
            dataset_case_free(listEntry->data);
        }
        list_freeList(dataset_version_snapshot->cases);
        dataset_version_snapshot->cases = NULL;
    }
    if (dataset_version_snapshot->corpus_root) {
        free(dataset_version_snapshot->corpus_root);
        dataset_version_snapshot->corpus_root = NULL;
    }
    if (dataset_version_snapshot->created_at) {
        free(dataset_version_snapshot->created_at);
        dataset_version_snapshot->created_at = NULL;
    }
    if (dataset_version_snapshot->dataset_id) {
        free(dataset_version_snapshot->dataset_id);
        dataset_version_snapshot->dataset_id = NULL;
    }
    if (dataset_version_snapshot->project_id) {
        free(dataset_version_snapshot->project_id);
        dataset_version_snapshot->project_id = NULL;
    }
    if (dataset_version_snapshot->tenant_id) {
        free(dataset_version_snapshot->tenant_id);
        dataset_version_snapshot->tenant_id = NULL;
    }
    if (dataset_version_snapshot->version_id) {
        free(dataset_version_snapshot->version_id);
        dataset_version_snapshot->version_id = NULL;
    }
    free(dataset_version_snapshot);
}

cJSON *dataset_version_snapshot_convertToJSON(dataset_version_snapshot_t *dataset_version_snapshot) {
    cJSON *item = cJSON_CreateObject();

    // dataset_version_snapshot->cases
    if (!dataset_version_snapshot->cases) {
        goto fail;
    }
    cJSON *cases = cJSON_AddArrayToObject(item, "cases");
    if(cases == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *casesListEntry;
    if (dataset_version_snapshot->cases) {
    list_ForEach(casesListEntry, dataset_version_snapshot->cases) {
    cJSON *itemLocal = dataset_case_convertToJSON(casesListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(cases, itemLocal);
    }
    }


    // dataset_version_snapshot->corpus_root
    if (!dataset_version_snapshot->corpus_root) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "corpus_root", dataset_version_snapshot->corpus_root) == NULL) {
    goto fail; //String
    }


    // dataset_version_snapshot->created_at
    if (!dataset_version_snapshot->created_at) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "created_at", dataset_version_snapshot->created_at) == NULL) {
    goto fail; //Date-Time
    }


    // dataset_version_snapshot->dataset_id
    if (!dataset_version_snapshot->dataset_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "dataset_id", dataset_version_snapshot->dataset_id) == NULL) {
    goto fail; //String
    }


    // dataset_version_snapshot->project_id
    if (!dataset_version_snapshot->project_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "project_id", dataset_version_snapshot->project_id) == NULL) {
    goto fail; //String
    }


    // dataset_version_snapshot->tenant_id
    if (!dataset_version_snapshot->tenant_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "tenant_id", dataset_version_snapshot->tenant_id) == NULL) {
    goto fail; //String
    }


    // dataset_version_snapshot->version_id
    if (!dataset_version_snapshot->version_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "version_id", dataset_version_snapshot->version_id) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

dataset_version_snapshot_t *dataset_version_snapshot_parseFromJSON(cJSON *dataset_version_snapshotJSON){

    dataset_version_snapshot_t *dataset_version_snapshot_local_var = NULL;

    // define the local list for dataset_version_snapshot->cases
    list_t *casesList = NULL;

    // dataset_version_snapshot->cases
    cJSON *cases = cJSON_GetObjectItemCaseSensitive(dataset_version_snapshotJSON, "cases");
    if (cJSON_IsNull(cases)) {
        cases = NULL;
    }
    if (!cases) {
        goto end;
    }

    
    cJSON *cases_local_nonprimitive = NULL;
    if(!cJSON_IsArray(cases)){
        goto end; //nonprimitive container
    }

    casesList = list_createList();

    cJSON_ArrayForEach(cases_local_nonprimitive,cases )
    {
        if(!cJSON_IsObject(cases_local_nonprimitive)){
            goto end;
        }
        dataset_case_t *casesItem = dataset_case_parseFromJSON(cases_local_nonprimitive);

        list_addElement(casesList, casesItem);
    }

    // dataset_version_snapshot->corpus_root
    cJSON *corpus_root = cJSON_GetObjectItemCaseSensitive(dataset_version_snapshotJSON, "corpus_root");
    if (cJSON_IsNull(corpus_root)) {
        corpus_root = NULL;
    }
    if (!corpus_root) {
        goto end;
    }

    
    if(!cJSON_IsString(corpus_root))
    {
    goto end; //String
    }

    // dataset_version_snapshot->created_at
    cJSON *created_at = cJSON_GetObjectItemCaseSensitive(dataset_version_snapshotJSON, "created_at");
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

    // dataset_version_snapshot->dataset_id
    cJSON *dataset_id = cJSON_GetObjectItemCaseSensitive(dataset_version_snapshotJSON, "dataset_id");
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

    // dataset_version_snapshot->project_id
    cJSON *project_id = cJSON_GetObjectItemCaseSensitive(dataset_version_snapshotJSON, "project_id");
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

    // dataset_version_snapshot->tenant_id
    cJSON *tenant_id = cJSON_GetObjectItemCaseSensitive(dataset_version_snapshotJSON, "tenant_id");
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

    // dataset_version_snapshot->version_id
    cJSON *version_id = cJSON_GetObjectItemCaseSensitive(dataset_version_snapshotJSON, "version_id");
    if (cJSON_IsNull(version_id)) {
        version_id = NULL;
    }
    if (!version_id) {
        goto end;
    }

    
    if(!cJSON_IsString(version_id))
    {
    goto end; //String
    }


    dataset_version_snapshot_local_var = dataset_version_snapshot_create_internal (
        casesList,
        strdup(corpus_root->valuestring),
        strdup(created_at->valuestring),
        strdup(dataset_id->valuestring),
        strdup(project_id->valuestring),
        strdup(tenant_id->valuestring),
        strdup(version_id->valuestring)
        );

    return dataset_version_snapshot_local_var;
end:
    if (casesList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, casesList) {
            dataset_case_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(casesList);
        casesList = NULL;
    }
    return NULL;

}
