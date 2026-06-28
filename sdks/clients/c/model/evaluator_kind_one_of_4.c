#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "evaluator_kind_one_of_4.h"


char* evaluator_kind_one_of_4_type_ToString(beater_api_evaluator_kind_one_of_4_TYPE_e type) {
    char* typeArray[] =  { "NULL", "cost_budget" };
    return typeArray[type];
}

beater_api_evaluator_kind_one_of_4_TYPE_e evaluator_kind_one_of_4_type_FromString(char* type){
    int stringToReturn = 0;
    char *typeArray[] =  { "NULL", "cost_budget" };
    size_t sizeofArray = sizeof(typeArray) / sizeof(typeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(type, typeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static evaluator_kind_one_of_4_t *evaluator_kind_one_of_4_create_internal(
    long max_micros,
    beater_api_evaluator_kind_one_of_4_TYPE_e type
    ) {
    evaluator_kind_one_of_4_t *evaluator_kind_one_of_4_local_var = malloc(sizeof(evaluator_kind_one_of_4_t));
    if (!evaluator_kind_one_of_4_local_var) {
        return NULL;
    }
    evaluator_kind_one_of_4_local_var->max_micros = max_micros;
    evaluator_kind_one_of_4_local_var->type = type;

    evaluator_kind_one_of_4_local_var->_library_owned = 1;
    return evaluator_kind_one_of_4_local_var;
}

__attribute__((deprecated)) evaluator_kind_one_of_4_t *evaluator_kind_one_of_4_create(
    long max_micros,
    beater_api_evaluator_kind_one_of_4_TYPE_e type
    ) {
    return evaluator_kind_one_of_4_create_internal (
        max_micros,
        type
        );
}

void evaluator_kind_one_of_4_free(evaluator_kind_one_of_4_t *evaluator_kind_one_of_4) {
    if(NULL == evaluator_kind_one_of_4){
        return ;
    }
    if(evaluator_kind_one_of_4->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "evaluator_kind_one_of_4_free");
        return ;
    }
    listEntry_t *listEntry;
    free(evaluator_kind_one_of_4);
}

cJSON *evaluator_kind_one_of_4_convertToJSON(evaluator_kind_one_of_4_t *evaluator_kind_one_of_4) {
    cJSON *item = cJSON_CreateObject();

    // evaluator_kind_one_of_4->max_micros
    if (!evaluator_kind_one_of_4->max_micros) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "max_micros", evaluator_kind_one_of_4->max_micros) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind_one_of_4->type
    if (beater_api_evaluator_kind_one_of_4_TYPE_NULL == evaluator_kind_one_of_4->type) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "type", evaluator_kind_one_of_4_type_ToString(evaluator_kind_one_of_4->type)) == NULL)
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

evaluator_kind_one_of_4_t *evaluator_kind_one_of_4_parseFromJSON(cJSON *evaluator_kind_one_of_4JSON){

    evaluator_kind_one_of_4_t *evaluator_kind_one_of_4_local_var = NULL;

    // evaluator_kind_one_of_4->max_micros
    cJSON *max_micros = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_4JSON, "max_micros");
    if (cJSON_IsNull(max_micros)) {
        max_micros = NULL;
    }
    if (!max_micros) {
        goto end;
    }

    
    if(!cJSON_IsNumber(max_micros))
    {
    goto end; //Numeric
    }

    // evaluator_kind_one_of_4->type
    cJSON *type = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_4JSON, "type");
    if (cJSON_IsNull(type)) {
        type = NULL;
    }
    if (!type) {
        goto end;
    }

    beater_api_evaluator_kind_one_of_4_TYPE_e typeVariable;
    
    if(!cJSON_IsString(type))
    {
    goto end; //Enum
    }
    typeVariable = evaluator_kind_one_of_4_type_FromString(type->valuestring);


    evaluator_kind_one_of_4_local_var = evaluator_kind_one_of_4_create_internal (
        max_micros->valuedouble,
        typeVariable
        );

    return evaluator_kind_one_of_4_local_var;
end:
    return NULL;

}
