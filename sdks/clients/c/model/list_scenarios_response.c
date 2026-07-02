#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "list_scenarios_response.h"



static list_scenarios_response_t *list_scenarios_response_create_internal(
    char *next_cursor,
    list_t *scenarios
    ) {
    list_scenarios_response_t *list_scenarios_response_local_var = malloc(sizeof(list_scenarios_response_t));
    if (!list_scenarios_response_local_var) {
        return NULL;
    }
    list_scenarios_response_local_var->next_cursor = next_cursor;
    list_scenarios_response_local_var->scenarios = scenarios;

    list_scenarios_response_local_var->_library_owned = 1;
    return list_scenarios_response_local_var;
}

__attribute__((deprecated)) list_scenarios_response_t *list_scenarios_response_create(
    char *next_cursor,
    list_t *scenarios
    ) {
    return list_scenarios_response_create_internal (
        next_cursor,
        scenarios
        );
}

void list_scenarios_response_free(list_scenarios_response_t *list_scenarios_response) {
    if(NULL == list_scenarios_response){
        return ;
    }
    if(list_scenarios_response->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "list_scenarios_response_free");
        return ;
    }
    listEntry_t *listEntry;
    if (list_scenarios_response->next_cursor) {
        free(list_scenarios_response->next_cursor);
        list_scenarios_response->next_cursor = NULL;
    }
    if (list_scenarios_response->scenarios) {
        list_ForEach(listEntry, list_scenarios_response->scenarios) {
            scenario_free(listEntry->data);
        }
        list_freeList(list_scenarios_response->scenarios);
        list_scenarios_response->scenarios = NULL;
    }
    free(list_scenarios_response);
}

cJSON *list_scenarios_response_convertToJSON(list_scenarios_response_t *list_scenarios_response) {
    cJSON *item = cJSON_CreateObject();

    // list_scenarios_response->next_cursor
    if(list_scenarios_response->next_cursor) {
    if(cJSON_AddStringToObject(item, "next_cursor", list_scenarios_response->next_cursor) == NULL) {
    goto fail; //String
    }
    }


    // list_scenarios_response->scenarios
    if (!list_scenarios_response->scenarios) {
        goto fail;
    }
    cJSON *scenarios = cJSON_AddArrayToObject(item, "scenarios");
    if(scenarios == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *scenariosListEntry;
    if (list_scenarios_response->scenarios) {
    list_ForEach(scenariosListEntry, list_scenarios_response->scenarios) {
    cJSON *itemLocal = scenario_convertToJSON(scenariosListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(scenarios, itemLocal);
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

list_scenarios_response_t *list_scenarios_response_parseFromJSON(cJSON *list_scenarios_responseJSON){

    list_scenarios_response_t *list_scenarios_response_local_var = NULL;

    // define the local list for list_scenarios_response->scenarios
    list_t *scenariosList = NULL;

    // list_scenarios_response->next_cursor
    cJSON *next_cursor = cJSON_GetObjectItemCaseSensitive(list_scenarios_responseJSON, "next_cursor");
    if (cJSON_IsNull(next_cursor)) {
        next_cursor = NULL;
    }
    if (next_cursor) { 
    if(!cJSON_IsString(next_cursor) && !cJSON_IsNull(next_cursor))
    {
    goto end; //String
    }
    }

    // list_scenarios_response->scenarios
    cJSON *scenarios = cJSON_GetObjectItemCaseSensitive(list_scenarios_responseJSON, "scenarios");
    if (cJSON_IsNull(scenarios)) {
        scenarios = NULL;
    }
    if (!scenarios) {
        goto end;
    }

    
    cJSON *scenarios_local_nonprimitive = NULL;
    if(!cJSON_IsArray(scenarios)){
        goto end; //nonprimitive container
    }

    scenariosList = list_createList();

    cJSON_ArrayForEach(scenarios_local_nonprimitive,scenarios )
    {
        if(!cJSON_IsObject(scenarios_local_nonprimitive)){
            goto end;
        }
        scenario_t *scenariosItem = scenario_parseFromJSON(scenarios_local_nonprimitive);

        list_addElement(scenariosList, scenariosItem);
    }


    list_scenarios_response_local_var = list_scenarios_response_create_internal (
        next_cursor && !cJSON_IsNull(next_cursor) ? strdup(next_cursor->valuestring) : NULL,
        scenariosList
        );

    return list_scenarios_response_local_var;
end:
    if (scenariosList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, scenariosList) {
            scenario_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(scenariosList);
        scenariosList = NULL;
    }
    return NULL;

}
