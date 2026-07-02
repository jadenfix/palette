#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "mine_scenarios_response.h"



static mine_scenarios_response_t *mine_scenarios_response_create_internal(
    list_t *clusters
    ) {
    mine_scenarios_response_t *mine_scenarios_response_local_var = malloc(sizeof(mine_scenarios_response_t));
    if (!mine_scenarios_response_local_var) {
        return NULL;
    }
    mine_scenarios_response_local_var->clusters = clusters;

    mine_scenarios_response_local_var->_library_owned = 1;
    return mine_scenarios_response_local_var;
}

__attribute__((deprecated)) mine_scenarios_response_t *mine_scenarios_response_create(
    list_t *clusters
    ) {
    return mine_scenarios_response_create_internal (
        clusters
        );
}

void mine_scenarios_response_free(mine_scenarios_response_t *mine_scenarios_response) {
    if(NULL == mine_scenarios_response){
        return ;
    }
    if(mine_scenarios_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "mine_scenarios_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (mine_scenarios_response->clusters) {
        list_ForEach(listEntry, mine_scenarios_response->clusters) {
            scenario_cluster_free(listEntry->data);
        }
        list_freeList(mine_scenarios_response->clusters);
        mine_scenarios_response->clusters = NULL;
    }
    free(mine_scenarios_response);
}

cJSON *mine_scenarios_response_convertToJSON(mine_scenarios_response_t *mine_scenarios_response) {
    cJSON *item = cJSON_CreateObject();

    // mine_scenarios_response->clusters
    if (!mine_scenarios_response->clusters) {
        goto fail;
    }
    cJSON *clusters = cJSON_AddArrayToObject(item, "clusters");
    if(clusters == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *clustersListEntry;
    if (mine_scenarios_response->clusters) {
    list_ForEach(clustersListEntry, mine_scenarios_response->clusters) {
    cJSON *itemLocal = scenario_cluster_convertToJSON(clustersListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(clusters, itemLocal);
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

mine_scenarios_response_t *mine_scenarios_response_parseFromJSON(cJSON *mine_scenarios_responseJSON){

    mine_scenarios_response_t *mine_scenarios_response_local_var = NULL;

    // define the local list for mine_scenarios_response->clusters
    list_t *clustersList = NULL;

    // mine_scenarios_response->clusters
    cJSON *clusters = cJSON_GetObjectItemCaseSensitive(mine_scenarios_responseJSON, "clusters");
    if (cJSON_IsNull(clusters)) {
        clusters = NULL;
    }
    if (!clusters) {
        goto end;
    }

    
    cJSON *clusters_local_nonprimitive = NULL;
    if(!cJSON_IsArray(clusters)){
        goto end; //nonprimitive container
    }

    clustersList = list_createList();

    cJSON_ArrayForEach(clusters_local_nonprimitive,clusters )
    {
        if(!cJSON_IsObject(clusters_local_nonprimitive)){
            goto end;
        }
        scenario_cluster_t *clustersItem = scenario_cluster_parseFromJSON(clusters_local_nonprimitive);

        list_addElement(clustersList, clustersItem);
    }


    mine_scenarios_response_local_var = mine_scenarios_response_create_internal (
        clustersList
        );

    return mine_scenarios_response_local_var;
end:
    if (clustersList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, clustersList) {
            scenario_cluster_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(clustersList);
        clustersList = NULL;
    }
    return NULL;

}
