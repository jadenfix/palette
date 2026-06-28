#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "evaluator_kind_one_of_7.h"


char* evaluator_kind_one_of_7_type_ToString(beater_api_evaluator_kind_one_of_7_TYPE_e type) {
    char* typeArray[] =  { "NULL", "browser_task_success" };
    return typeArray[type];
}

beater_api_evaluator_kind_one_of_7_TYPE_e evaluator_kind_one_of_7_type_FromString(char* type){
    int stringToReturn = 0;
    char *typeArray[] =  { "NULL", "browser_task_success" };
    size_t sizeofArray = sizeof(typeArray) / sizeof(typeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(type, typeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static evaluator_kind_one_of_7_t *evaluator_kind_one_of_7_create_internal(
    char *dom_contains,
    beater_api_evaluator_kind_one_of_7_TYPE_e type,
    char *url_contains
    ) {
    evaluator_kind_one_of_7_t *evaluator_kind_one_of_7_local_var = malloc(sizeof(evaluator_kind_one_of_7_t));
    if (!evaluator_kind_one_of_7_local_var) {
        return NULL;
    }
    evaluator_kind_one_of_7_local_var->dom_contains = dom_contains;
    evaluator_kind_one_of_7_local_var->type = type;
    evaluator_kind_one_of_7_local_var->url_contains = url_contains;

    evaluator_kind_one_of_7_local_var->_library_owned = 1;
    return evaluator_kind_one_of_7_local_var;
}

__attribute__((deprecated)) evaluator_kind_one_of_7_t *evaluator_kind_one_of_7_create(
    char *dom_contains,
    beater_api_evaluator_kind_one_of_7_TYPE_e type,
    char *url_contains
    ) {
    return evaluator_kind_one_of_7_create_internal (
        dom_contains,
        type,
        url_contains
        );
}

void evaluator_kind_one_of_7_free(evaluator_kind_one_of_7_t *evaluator_kind_one_of_7) {
    if(NULL == evaluator_kind_one_of_7){
        return ;
    }
    if(evaluator_kind_one_of_7->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "evaluator_kind_one_of_7_free");
        return ;
    }
    listEntry_t *listEntry;
    if (evaluator_kind_one_of_7->dom_contains) {
        free(evaluator_kind_one_of_7->dom_contains);
        evaluator_kind_one_of_7->dom_contains = NULL;
    }
    if (evaluator_kind_one_of_7->url_contains) {
        free(evaluator_kind_one_of_7->url_contains);
        evaluator_kind_one_of_7->url_contains = NULL;
    }
    free(evaluator_kind_one_of_7);
}

cJSON *evaluator_kind_one_of_7_convertToJSON(evaluator_kind_one_of_7_t *evaluator_kind_one_of_7) {
    cJSON *item = cJSON_CreateObject();

    // evaluator_kind_one_of_7->dom_contains
    if(evaluator_kind_one_of_7->dom_contains) {
    if(cJSON_AddStringToObject(item, "dom_contains", evaluator_kind_one_of_7->dom_contains) == NULL) {
    goto fail; //String
    }
    }


    // evaluator_kind_one_of_7->type
    if (beater_api_evaluator_kind_one_of_7_TYPE_NULL == evaluator_kind_one_of_7->type) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "type", evaluator_kind_one_of_7_type_ToString(evaluator_kind_one_of_7->type)) == NULL)
    {
    goto fail; //Enum
    }


    // evaluator_kind_one_of_7->url_contains
    if(evaluator_kind_one_of_7->url_contains) {
    if(cJSON_AddStringToObject(item, "url_contains", evaluator_kind_one_of_7->url_contains) == NULL) {
    goto fail; //String
    }
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

evaluator_kind_one_of_7_t *evaluator_kind_one_of_7_parseFromJSON(cJSON *evaluator_kind_one_of_7JSON){

    evaluator_kind_one_of_7_t *evaluator_kind_one_of_7_local_var = NULL;

    // evaluator_kind_one_of_7->dom_contains
    cJSON *dom_contains = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_7JSON, "dom_contains");
    if (cJSON_IsNull(dom_contains)) {
        dom_contains = NULL;
    }
    if (dom_contains) { 
    if(!cJSON_IsString(dom_contains) && !cJSON_IsNull(dom_contains))
    {
    goto end; //String
    }
    }

    // evaluator_kind_one_of_7->type
    cJSON *type = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_7JSON, "type");
    if (cJSON_IsNull(type)) {
        type = NULL;
    }
    if (!type) {
        goto end;
    }

    beater_api_evaluator_kind_one_of_7_TYPE_e typeVariable;
    
    if(!cJSON_IsString(type))
    {
    goto end; //Enum
    }
    typeVariable = evaluator_kind_one_of_7_type_FromString(type->valuestring);

    // evaluator_kind_one_of_7->url_contains
    cJSON *url_contains = cJSON_GetObjectItemCaseSensitive(evaluator_kind_one_of_7JSON, "url_contains");
    if (cJSON_IsNull(url_contains)) {
        url_contains = NULL;
    }
    if (url_contains) { 
    if(!cJSON_IsString(url_contains) && !cJSON_IsNull(url_contains))
    {
    goto end; //String
    }
    }


    evaluator_kind_one_of_7_local_var = evaluator_kind_one_of_7_create_internal (
        dom_contains && !cJSON_IsNull(dom_contains) ? strdup(dom_contains->valuestring) : NULL,
        typeVariable,
        url_contains && !cJSON_IsNull(url_contains) ? strdup(url_contains->valuestring) : NULL
        );

    return evaluator_kind_one_of_7_local_var;
end:
    return NULL;

}
