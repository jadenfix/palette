#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "model_routing_one_of.h"


char* model_routing_one_of_kind_ToString(beater_api_model_routing_one_of_KIND_e kind) {
    char* kindArray[] =  { "NULL", "byok" };
    return kindArray[kind];
}

beater_api_model_routing_one_of_KIND_e model_routing_one_of_kind_FromString(char* kind){
    int stringToReturn = 0;
    char *kindArray[] =  { "NULL", "byok" };
    size_t sizeofArray = sizeof(kindArray) / sizeof(kindArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(kind, kindArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static model_routing_one_of_t *model_routing_one_of_create_internal(
    beater_api_model_routing_one_of_KIND_e kind,
    char *provider_secret_id
    ) {
    model_routing_one_of_t *model_routing_one_of_local_var = malloc(sizeof(model_routing_one_of_t));
    if (!model_routing_one_of_local_var) {
        return NULL;
    }
    model_routing_one_of_local_var->kind = kind;
    model_routing_one_of_local_var->provider_secret_id = provider_secret_id;

    model_routing_one_of_local_var->_library_owned = 1;
    return model_routing_one_of_local_var;
}

__attribute__((deprecated)) model_routing_one_of_t *model_routing_one_of_create(
    beater_api_model_routing_one_of_KIND_e kind,
    char *provider_secret_id
    ) {
    return model_routing_one_of_create_internal (
        kind,
        provider_secret_id
        );
}

void model_routing_one_of_free(model_routing_one_of_t *model_routing_one_of) {
    if(NULL == model_routing_one_of){
        return ;
    }
    if(model_routing_one_of->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "model_routing_one_of_free");
        return ;
    }
    listEntry_t *listEntry;
    if (model_routing_one_of->provider_secret_id) {
        free(model_routing_one_of->provider_secret_id);
        model_routing_one_of->provider_secret_id = NULL;
    }
    free(model_routing_one_of);
}

cJSON *model_routing_one_of_convertToJSON(model_routing_one_of_t *model_routing_one_of) {
    cJSON *item = cJSON_CreateObject();

    // model_routing_one_of->kind
    if (beater_api_model_routing_one_of_KIND_NULL == model_routing_one_of->kind) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "kind", model_routing_one_of_kind_ToString(model_routing_one_of->kind)) == NULL)
    {
    goto fail; //Enum
    }


    // model_routing_one_of->provider_secret_id
    if (!model_routing_one_of->provider_secret_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "provider_secret_id", model_routing_one_of->provider_secret_id) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

model_routing_one_of_t *model_routing_one_of_parseFromJSON(cJSON *model_routing_one_ofJSON){

    model_routing_one_of_t *model_routing_one_of_local_var = NULL;

    // model_routing_one_of->kind
    cJSON *kind = cJSON_GetObjectItemCaseSensitive(model_routing_one_ofJSON, "kind");
    if (cJSON_IsNull(kind)) {
        kind = NULL;
    }
    if (!kind) {
        goto end;
    }

    beater_api_model_routing_one_of_KIND_e kindVariable;
    
    if(!cJSON_IsString(kind))
    {
    goto end; //Enum
    }
    kindVariable = model_routing_one_of_kind_FromString(kind->valuestring);

    // model_routing_one_of->provider_secret_id
    cJSON *provider_secret_id = cJSON_GetObjectItemCaseSensitive(model_routing_one_ofJSON, "provider_secret_id");
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


    model_routing_one_of_local_var = model_routing_one_of_create_internal (
        kindVariable,
        strdup(provider_secret_id->valuestring)
        );

    return model_routing_one_of_local_var;
end:
    return NULL;

}
