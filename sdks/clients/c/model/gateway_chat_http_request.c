#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "gateway_chat_http_request.h"



static gateway_chat_http_request_t *gateway_chat_http_request_create_internal(
    long budget_micros,
    chat_completion_request_t *completion,
    list_t *routing
    ) {
    gateway_chat_http_request_t *gateway_chat_http_request_local_var = malloc(sizeof(gateway_chat_http_request_t));
    if (!gateway_chat_http_request_local_var) {
        return NULL;
    }
    gateway_chat_http_request_local_var->budget_micros = budget_micros;
    gateway_chat_http_request_local_var->completion = completion;
    gateway_chat_http_request_local_var->routing = routing;

    gateway_chat_http_request_local_var->_library_owned = 1;
    return gateway_chat_http_request_local_var;
}

__attribute__((deprecated)) gateway_chat_http_request_t *gateway_chat_http_request_create(
    long budget_micros,
    chat_completion_request_t *completion,
    list_t *routing
    ) {
    return gateway_chat_http_request_create_internal (
        budget_micros,
        completion,
        routing
        );
}

void gateway_chat_http_request_free(gateway_chat_http_request_t *gateway_chat_http_request) {
    if(NULL == gateway_chat_http_request){
        return ;
    }
    if(gateway_chat_http_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "gateway_chat_http_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (gateway_chat_http_request->completion) {
        chat_completion_request_free(gateway_chat_http_request->completion);
        gateway_chat_http_request->completion = NULL;
    }
    if (gateway_chat_http_request->routing) {
        list_ForEach(listEntry, gateway_chat_http_request->routing) {
            model_routing_free(listEntry->data);
        }
        list_freeList(gateway_chat_http_request->routing);
        gateway_chat_http_request->routing = NULL;
    }
    free(gateway_chat_http_request);
}

cJSON *gateway_chat_http_request_convertToJSON(gateway_chat_http_request_t *gateway_chat_http_request) {
    cJSON *item = cJSON_CreateObject();

    // gateway_chat_http_request->budget_micros
    if(gateway_chat_http_request->budget_micros) {
    if(cJSON_AddNumberToObject(item, "budget_micros", gateway_chat_http_request->budget_micros) == NULL) {
    goto fail; //Numeric
    }
    }


    // gateway_chat_http_request->completion
    if (!gateway_chat_http_request->completion) {
        goto fail;
    }
    cJSON *completion_local_JSON = chat_completion_request_convertToJSON(gateway_chat_http_request->completion);
    if(completion_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "completion", completion_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // gateway_chat_http_request->routing
    if(gateway_chat_http_request->routing) {
    cJSON *routing = cJSON_AddArrayToObject(item, "routing");
    if(routing == NULL) {
    goto fail; //nonprimitive container
    }

    listEntry_t *routingListEntry;
    if (gateway_chat_http_request->routing) {
    list_ForEach(routingListEntry, gateway_chat_http_request->routing) {
    cJSON *itemLocal = model_routing_convertToJSON(routingListEntry->data);
    if(itemLocal == NULL) {
    goto fail;
    }
    cJSON_AddItemToArray(routing, itemLocal);
    }
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

gateway_chat_http_request_t *gateway_chat_http_request_parseFromJSON(cJSON *gateway_chat_http_requestJSON){

    gateway_chat_http_request_t *gateway_chat_http_request_local_var = NULL;

    // define the local variable for gateway_chat_http_request->completion
    chat_completion_request_t *completion_local_nonprim = NULL;

    // define the local list for gateway_chat_http_request->routing
    list_t *routingList = NULL;

    // gateway_chat_http_request->budget_micros
    cJSON *budget_micros = cJSON_GetObjectItemCaseSensitive(gateway_chat_http_requestJSON, "budget_micros");
    if (cJSON_IsNull(budget_micros)) {
        budget_micros = NULL;
    }
    if (budget_micros) { 
    if(!cJSON_IsNumber(budget_micros))
    {
    goto end; //Numeric
    }
    }

    // gateway_chat_http_request->completion
    cJSON *completion = cJSON_GetObjectItemCaseSensitive(gateway_chat_http_requestJSON, "completion");
    if (cJSON_IsNull(completion)) {
        completion = NULL;
    }
    if (!completion) {
        goto end;
    }

    
    completion_local_nonprim = chat_completion_request_parseFromJSON(completion); //nonprimitive

    // gateway_chat_http_request->routing
    cJSON *routing = cJSON_GetObjectItemCaseSensitive(gateway_chat_http_requestJSON, "routing");
    if (cJSON_IsNull(routing)) {
        routing = NULL;
    }
    if (routing) { 
    cJSON *routing_local_nonprimitive = NULL;
    if(!cJSON_IsArray(routing)){
        goto end; //nonprimitive container
    }

    routingList = list_createList();

    cJSON_ArrayForEach(routing_local_nonprimitive,routing )
    {
        if(!cJSON_IsObject(routing_local_nonprimitive)){
            goto end;
        }
        model_routing_t *routingItem = model_routing_parseFromJSON(routing_local_nonprimitive);

        list_addElement(routingList, routingItem);
    }
    }


    gateway_chat_http_request_local_var = gateway_chat_http_request_create_internal (
        budget_micros ? budget_micros->valuedouble : 0,
        completion_local_nonprim,
        routing ? routingList : NULL
        );

    return gateway_chat_http_request_local_var;
end:
    if (completion_local_nonprim) {
        chat_completion_request_free(completion_local_nonprim);
        completion_local_nonprim = NULL;
    }
    if (routingList) {
        listEntry_t *listEntry = NULL;
        list_ForEach(listEntry, routingList) {
            model_routing_free(listEntry->data);
            listEntry->data = NULL;
        }
        list_freeList(routingList);
        routingList = NULL;
    }
    return NULL;

}
