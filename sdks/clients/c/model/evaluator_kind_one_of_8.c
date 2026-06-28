#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "evaluator_kind_one_of_8.h"


char* evaluator_kind_one_of_8_type_ToString(beater_api_evaluator_kind_one_of_8_TYPE_e type) {
    char* typeArray[] =  { "NULL", "browser_step_efficiency" };
    return typeArray[type];
}

beater_api_evaluator_kind_one_of_8_TYPE_e evaluator_kind_one_of_8_type_FromString(char* type){
    int stringToReturn = 0;
    char *typeArray[] =  { "NULL", "browser_step_efficiency" };
    size_t sizeofArray = sizeof(typeArray) / sizeof(typeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(type, typeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static evaluator_kind_one_of_8_t *evaluator_kind_one_of_8_create_internal(
    long max_steps,
    beater_api_evaluator_kind_one_of_8_TYPE_e type
    ) {
    evaluator_kind_one_of_8_t *evaluator_kind_one_of_8_local_var = malloc(sizeof(evaluator_kind_one_of_8_t));
    if (!evaluator_kind_one_of_8_local_var) {
        return NULL;
    }
    evaluator_kind_one_of_8_local_var->max_steps = max_steps;
    evaluator_kind_one_of_8_local_var->type = type;

    evaluator_kind_one_of_8_local_var->_library_owned = 1;
    return evaluator_kind_one_of_8_local_var;
}

__attribute__((deprecated)) evaluator_kind_one_of_8_t *evaluator_kind_one_of_8_create(
    long max_steps,
    beater_api_evaluator_kind_one_of_8_TYPE_e type
    ) {
    return evaluator_kind_one_of_8_create_internal (
        max_steps,
        type
        );
}

void evaluator_kind_one_of_8_free(evaluator_kind_one_of_8_t *evaluator_kind_one_of_8) {
    if(NULL == evaluator_kind_one_of_8){
        return ;
    }
    if(evaluator_kind_one_of_8->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "evaluator_kind_one_of_8_free");
        return ;
    }
    listEntry_t *listEntry;
    free(evaluator_kind_one_of_8);
}

cJSON *evaluator_kind_one_of_8_convertToJSON(evaluator_kind_one_of_8_t *evaluator_kind_one_of_8) {
    cJSON *item = cJSON_CreateObject();

    // evaluator_kind_one_of_8->max_steps
    if (!evaluator_kind_one_of_8->max_steps) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "max_steps", evaluator_kind_one_of_8->max_steps) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind_one_of_8->type
    if (beater_api_evaluator_kind_one_of_8_TYPE_NULL == evaluator_kind_one_of_8->type) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "type", evaluator_kind_one_of_8_type_ToString(evaluator_kind_one_of_8->type)) == NULL)
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

evaluator_kind_one_of_8_t *evaluator_kind_one_of_8_parseFromJSON(cJSON *evaluator_kind_one_of_8JSON){

    evaluator_kind_one_of_8_t *evaluator_kind_one_of_8_local_var = NULL;

    // evaluator_kind_one_of_8->max_steps
    cJSON *max_steps = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_8JSON, "max_steps");
    if (cJSON_IsNull(max_steps)) {
        max_steps = NULL;
    }
    if (!max_steps) {
        goto end;
    }

    
    if(!cJSON_IsNumber(max_steps))
    {
    goto end; //Numeric
    }

    // evaluator_kind_one_of_8->type
    cJSON *type = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_8JSON, "type");
    if (cJSON_IsNull(type)) {
        type = NULL;
    }
    if (!type) {
        goto end;
    }

    beater_api_evaluator_kind_one_of_8_TYPE_e typeVariable;
    
    if(!cJSON_IsString(type))
    {
    goto end; //Enum
    }
    typeVariable = evaluator_kind_one_of_8_type_FromString(type->valuestring);


    evaluator_kind_one_of_8_local_var = evaluator_kind_one_of_8_create_internal (
        max_steps->valuedouble,
        typeVariable
        );

    return evaluator_kind_one_of_8_local_var;
end:
    return NULL;

}
