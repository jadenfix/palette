#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "evaluator_kind_one_of_3.h"


char* evaluator_kind_one_of_3_type_ToString(beater_api_evaluator_kind_one_of_3_TYPE_e type) {
    char* typeArray[] =  { "NULL", "json_object" };
    return typeArray[type];
}

beater_api_evaluator_kind_one_of_3_TYPE_e evaluator_kind_one_of_3_type_FromString(char* type){
    int stringToReturn = 0;
    char *typeArray[] =  { "NULL", "json_object" };
    size_t sizeofArray = sizeof(typeArray) / sizeof(typeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(type, typeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static evaluator_kind_one_of_3_t *evaluator_kind_one_of_3_create_internal(
    beater_api_evaluator_kind_one_of_3_TYPE_e type
    ) {
    evaluator_kind_one_of_3_t *evaluator_kind_one_of_3_local_var = malloc(sizeof(evaluator_kind_one_of_3_t));
    if (!evaluator_kind_one_of_3_local_var) {
        return NULL;
    }
    evaluator_kind_one_of_3_local_var->type = type;

    evaluator_kind_one_of_3_local_var->_library_owned = 1;
    return evaluator_kind_one_of_3_local_var;
}

__attribute__((deprecated)) evaluator_kind_one_of_3_t *evaluator_kind_one_of_3_create(
    beater_api_evaluator_kind_one_of_3_TYPE_e type
    ) {
    return evaluator_kind_one_of_3_create_internal (
        type
        );
}

void evaluator_kind_one_of_3_free(evaluator_kind_one_of_3_t *evaluator_kind_one_of_3) {
    if(NULL == evaluator_kind_one_of_3){
        return ;
    }
    if(evaluator_kind_one_of_3->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "evaluator_kind_one_of_3_free");
        return ;
    }
    listEntry_t *listEntry;
    free(evaluator_kind_one_of_3);
}

cJSON *evaluator_kind_one_of_3_convertToJSON(evaluator_kind_one_of_3_t *evaluator_kind_one_of_3) {
    cJSON *item = cJSON_CreateObject();

    // evaluator_kind_one_of_3->type
    if (beater_api_evaluator_kind_one_of_3_TYPE_NULL == evaluator_kind_one_of_3->type) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "type", evaluator_kind_one_of_3_type_ToString(evaluator_kind_one_of_3->type)) == NULL)
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

evaluator_kind_one_of_3_t *evaluator_kind_one_of_3_parseFromJSON(cJSON *evaluator_kind_one_of_3JSON){

    evaluator_kind_one_of_3_t *evaluator_kind_one_of_3_local_var = NULL;

    // evaluator_kind_one_of_3->type
    cJSON *type = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_3JSON, "type");
    if (cJSON_IsNull(type)) {
        type = NULL;
    }
    if (!type) {
        goto end;
    }

    beater_api_evaluator_kind_one_of_3_TYPE_e typeVariable;
    
    if(!cJSON_IsString(type))
    {
    goto end; //Enum
    }
    typeVariable = evaluator_kind_one_of_3_type_FromString(type->valuestring);


    evaluator_kind_one_of_3_local_var = evaluator_kind_one_of_3_create_internal (
        typeVariable
        );

    return evaluator_kind_one_of_3_local_var;
end:
    return NULL;

}
