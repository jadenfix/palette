#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "model_routing.h"


char* model_routing_kind_ToString(beater_api_model_routing_KIND_e kind) {
    char* kindArray[] =  { "NULL", "managed" };
    return kindArray[kind];
}

beater_api_model_routing_KIND_e model_routing_kind_FromString(char* kind){
    int stringToReturn = 0;
    char *kindArray[] =  { "NULL", "managed" };
    size_t sizeofArray = sizeof(kindArray) / sizeof(kindArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(kind, kindArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static model_routing_t *model_routing_create_internal(
    beater_api_model_routing_KIND_e kind,
    char *provider_secret_id,
    model_ref_t *default_model
    ) {
    model_routing_t *model_routing_local_var = malloc(sizeof(model_routing_t));
    if (!model_routing_local_var) {
        return NULL;
    }
    model_routing_local_var->kind = kind;
    model_routing_local_var->provider_secret_id = provider_secret_id;
    model_routing_local_var->default_model = default_model;

    model_routing_local_var->_library_owned = 1;
    return model_routing_local_var;
}

__attribute__((deprecated)) model_routing_t *model_routing_create(
    beater_api_model_routing_KIND_e kind,
    char *provider_secret_id,
    model_ref_t *default_model
    ) {
    return model_routing_create_internal (
        kind,
        provider_secret_id,
        default_model
        );
}

void model_routing_free(model_routing_t *model_routing) {
    if(NULL == model_routing){
        return ;
    }
    if(model_routing->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "model_routing_free");
        return ;
    }
    listEntry_t *listEntry;
    if (model_routing->provider_secret_id) {
        free(model_routing->provider_secret_id);
        model_routing->provider_secret_id = NULL;
    }
    if (model_routing->default_model) {
        model_ref_free(model_routing->default_model);
        model_routing->default_model = NULL;
    }
    free(model_routing);
}

cJSON *model_routing_convertToJSON(model_routing_t *model_routing) {
    cJSON *item = cJSON_CreateObject();

    // model_routing->kind
    if (beater_api_model_routing_KIND_NULL == model_routing->kind) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "kind", model_routing_kind_ToString(model_routing->kind)) == NULL)
    {
    goto fail; //Enum
    }


    // model_routing->provider_secret_id
    if (!model_routing->provider_secret_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "provider_secret_id", model_routing->provider_secret_id) == NULL) {
    goto fail; //String
    }


    // model_routing->default_model
    if (!model_routing->default_model) {
        goto fail;
    }
    cJSON *default_model_local_JSON = model_ref_convertToJSON(model_routing->default_model);
    if(default_model_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "default_model", default_model_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

model_routing_t *model_routing_parseFromJSON(cJSON *model_routingJSON){

    model_routing_t *model_routing_local_var = NULL;

    // define the local variable for model_routing->default_model
    model_ref_t *default_model_local_nonprim = NULL;

    // model_routing->kind
    cJSON *kind = cJSON_GetObjectItemCaseSensitive(model_routingJSON, "kind");
    if (cJSON_IsNull(kind)) {
        kind = NULL;
    }
    if (!kind) {
        goto end;
    }

    beater_api_model_routing_KIND_e kindVariable;
    
    if(!cJSON_IsString(kind))
    {
    goto end; //Enum
    }
    kindVariable = model_routing_kind_FromString(kind->valuestring);

    // model_routing->provider_secret_id
    cJSON *provider_secret_id = cJSON_GetObjectItemCaseSensitive(model_routingJSON, "provider_secret_id");
    if (cJSON_IsNull(provider_secret_id)) {
        provider_secret_id = NULL;
    }
    if (!provider_secret_id) {
        goto end;
    }

    
    if(!cJSON_IsString(provider_secret_id))
    {
    goto end; //String
    }

    // model_routing->default_model
    cJSON *default_model = cJSON_GetObjectItemCaseSensitive(model_routingJSON, "default_model");
    if (cJSON_IsNull(default_model)) {
        default_model = NULL;
    }
    if (!default_model) {
        goto end;
    }

    
    default_model_local_nonprim = model_ref_parseFromJSON(default_model); //nonprimitive


    model_routing_local_var = model_routing_create_internal (
        kindVariable,
        strdup(provider_secret_id->valuestring),
        default_model_local_nonprim
        );

    return model_routing_local_var;
end:
    if (default_model_local_nonprim) {
        model_ref_free(default_model_local_nonprim);
        default_model_local_nonprim = NULL;
    }
    return NULL;

}
