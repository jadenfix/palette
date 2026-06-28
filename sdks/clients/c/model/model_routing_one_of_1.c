#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "model_routing_one_of_1.h"


char* model_routing_one_of_1_kind_ToString(beater_api_model_routing_one_of_1_KIND_e kind) {
    char* kindArray[] =  { "NULL", "managed" };
    return kindArray[kind];
}

beater_api_model_routing_one_of_1_KIND_e model_routing_one_of_1_kind_FromString(char* kind){
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

static model_routing_one_of_1_t *model_routing_one_of_1_create_internal(
    model_ref_t *default_model,
    beater_api_model_routing_one_of_1_KIND_e kind
    ) {
    model_routing_one_of_1_t *model_routing_one_of_1_local_var = malloc(sizeof(model_routing_one_of_1_t));
    if (!model_routing_one_of_1_local_var) {
        return NULL;
    }
    model_routing_one_of_1_local_var->default_model = default_model;
    model_routing_one_of_1_local_var->kind = kind;

    model_routing_one_of_1_local_var->_library_owned = 1;
    return model_routing_one_of_1_local_var;
}

__attribute__((deprecated)) model_routing_one_of_1_t *model_routing_one_of_1_create(
    model_ref_t *default_model,
    beater_api_model_routing_one_of_1_KIND_e kind
    ) {
    return model_routing_one_of_1_create_internal (
        default_model,
        kind
        );
}

void model_routing_one_of_1_free(model_routing_one_of_1_t *model_routing_one_of_1) {
    if(NULL == model_routing_one_of_1){
        return ;
    }
    if(model_routing_one_of_1->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "model_routing_one_of_1_free");
        return ;
    }
    listEntry_t *listEntry;
    if (model_routing_one_of_1->default_model) {
        model_ref_free(model_routing_one_of_1->default_model);
        model_routing_one_of_1->default_model = NULL;
    }
    free(model_routing_one_of_1);
}

cJSON *model_routing_one_of_1_convertToJSON(model_routing_one_of_1_t *model_routing_one_of_1) {
    cJSON *item = cJSON_CreateObject();

    // model_routing_one_of_1->default_model
    if (!model_routing_one_of_1->default_model) {
        goto fail;
    }
    cJSON *default_model_local_JSON = model_ref_convertToJSON(model_routing_one_of_1->default_model);
    if(default_model_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "default_model", default_model_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // model_routing_one_of_1->kind
    if (beater_api_model_routing_one_of_1_KIND_NULL == model_routing_one_of_1->kind) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "kind", model_routing_one_of_1_kind_ToString(model_routing_one_of_1->kind)) == NULL)
    {
    goto fail; //Enum
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

model_routing_one_of_1_t *model_routing_one_of_1_parseFromJSON(cJSON *model_routing_one_of_1JSON){

    model_routing_one_of_1_t *model_routing_one_of_1_local_var = NULL;

    // define the local variable for model_routing_one_of_1->default_model
    model_ref_t *default_model_local_nonprim = NULL;

    // model_routing_one_of_1->default_model
    cJSON *default_model = cJSON_GetObjectItemCaseSensitive(model_routing_one_of_1JSON, "default_model");
    if (cJSON_IsNull(default_model)) {
        default_model = NULL;
    }
    if (!default_model) {
        goto end;
    }

    
    default_model_local_nonprim = model_ref_parseFromJSON(default_model); //nonprimitive

    // model_routing_one_of_1->kind
    cJSON *kind = cJSON_GetObjectItemCaseSensitive(model_routing_one_of_1JSON, "kind");
    if (cJSON_IsNull(kind)) {
        kind = NULL;
    }
    if (!kind) {
        goto end;
    }

    beater_api_model_routing_one_of_1_KIND_e kindVariable;
    
    if(!cJSON_IsString(kind))
    {
    goto end; //Enum
    }
    kindVariable = model_routing_one_of_1_kind_FromString(kind->valuestring);


    model_routing_one_of_1_local_var = model_routing_one_of_1_create_internal (
        default_model_local_nonprim,
        kindVariable
        );

    return model_routing_one_of_1_local_var;
end:
    if (default_model_local_nonprim) {
        model_ref_free(default_model_local_nonprim);
        default_model_local_nonprim = NULL;
    }
    return NULL;

}
