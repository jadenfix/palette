#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "evaluator_kind_one_of_2.h"


char* evaluator_kind_one_of_2_type_ToString(beater_api_evaluator_kind_one_of_2_TYPE_e type) {
    char* typeArray[] =  { "NULL", "numeric_tolerance" };
    return typeArray[type];
}

beater_api_evaluator_kind_one_of_2_TYPE_e evaluator_kind_one_of_2_type_FromString(char* type){
    int stringToReturn = 0;
    char *typeArray[] =  { "NULL", "numeric_tolerance" };
    size_t sizeofArray = sizeof(typeArray) / sizeof(typeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(type, typeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static evaluator_kind_one_of_2_t *evaluator_kind_one_of_2_create_internal(
    double abs,
    double rel,
    beater_api_evaluator_kind_one_of_2_TYPE_e type
    ) {
    evaluator_kind_one_of_2_t *evaluator_kind_one_of_2_local_var = malloc(sizeof(evaluator_kind_one_of_2_t));
    if (!evaluator_kind_one_of_2_local_var) {
        return NULL;
    }
    evaluator_kind_one_of_2_local_var->abs = abs;
    evaluator_kind_one_of_2_local_var->rel = rel;
    evaluator_kind_one_of_2_local_var->type = type;

    evaluator_kind_one_of_2_local_var->_library_owned = 1;
    return evaluator_kind_one_of_2_local_var;
}

__attribute__((deprecated)) evaluator_kind_one_of_2_t *evaluator_kind_one_of_2_create(
    double abs,
    double rel,
    beater_api_evaluator_kind_one_of_2_TYPE_e type
    ) {
    return evaluator_kind_one_of_2_create_internal (
        abs,
        rel,
        type
        );
}

void evaluator_kind_one_of_2_free(evaluator_kind_one_of_2_t *evaluator_kind_one_of_2) {
    if(NULL == evaluator_kind_one_of_2){
        return ;
    }
    if(evaluator_kind_one_of_2->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "evaluator_kind_one_of_2_free");
        return ;
    }
    listEntry_t *listEntry;
    free(evaluator_kind_one_of_2);
}

cJSON *evaluator_kind_one_of_2_convertToJSON(evaluator_kind_one_of_2_t *evaluator_kind_one_of_2) {
    cJSON *item = cJSON_CreateObject();

    // evaluator_kind_one_of_2->abs
    if (!evaluator_kind_one_of_2->abs) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "abs", evaluator_kind_one_of_2->abs) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind_one_of_2->rel
    if (!evaluator_kind_one_of_2->rel) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "rel", evaluator_kind_one_of_2->rel) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind_one_of_2->type
    if (beater_api_evaluator_kind_one_of_2_TYPE_NULL == evaluator_kind_one_of_2->type) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "type", evaluator_kind_one_of_2_type_ToString(evaluator_kind_one_of_2->type)) == NULL)
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

evaluator_kind_one_of_2_t *evaluator_kind_one_of_2_parseFromJSON(cJSON *evaluator_kind_one_of_2JSON){

    evaluator_kind_one_of_2_t *evaluator_kind_one_of_2_local_var = NULL;

    // evaluator_kind_one_of_2->abs
    cJSON *abs = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_2JSON, "abs");
    if (cJSON_IsNull(abs)) {
        abs = NULL;
    }
    if (!abs) {
        goto end;
    }

    
    if(!cJSON_IsNumber(abs))
    {
    goto end; //Numeric
    }

    // evaluator_kind_one_of_2->rel
    cJSON *rel = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_2JSON, "rel");
    if (cJSON_IsNull(rel)) {
        rel = NULL;
    }
    if (!rel) {
        goto end;
    }

    
    if(!cJSON_IsNumber(rel))
    {
    goto end; //Numeric
    }

    // evaluator_kind_one_of_2->type
    cJSON *type = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_2JSON, "type");
    if (cJSON_IsNull(type)) {
        type = NULL;
    }
    if (!type) {
        goto end;
    }

    beater_api_evaluator_kind_one_of_2_TYPE_e typeVariable;
    
    if(!cJSON_IsString(type))
    {
    goto end; //Enum
    }
    typeVariable = evaluator_kind_one_of_2_type_FromString(type->valuestring);


    evaluator_kind_one_of_2_local_var = evaluator_kind_one_of_2_create_internal (
        abs->valuedouble,
        rel->valuedouble,
        typeVariable
        );

    return evaluator_kind_one_of_2_local_var;
end:
    return NULL;

}
