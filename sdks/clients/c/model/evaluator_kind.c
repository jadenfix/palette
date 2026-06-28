#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "evaluator_kind.h"


char* evaluator_kind_type_ToString(beater_api_evaluator_kind_TYPE_e type) {
    char* typeArray[] =  { "NULL", "exact_match", "regex_match", "numeric_tolerance", "json_object", "cost_budget", "latency_budget_ms", "llm_judge", "browser_task_success", "browser_step_efficiency", "browser_grounding", "browser_recovery" };
    return typeArray[type];
}

beater_api_evaluator_kind_TYPE_e evaluator_kind_type_FromString(char* type){
    int stringToReturn = 0;
    char *typeArray[] =  { "NULL", "exact_match", "regex_match", "numeric_tolerance", "json_object", "cost_budget", "latency_budget_ms", "llm_judge", "browser_task_success", "browser_step_efficiency", "browser_grounding", "browser_recovery" };
    size_t sizeofArray = sizeof(typeArray) / sizeof(typeArray[0]);
    while(stringToReturn < sizeofArray) {
        if(strcmp(type, typeArray[stringToReturn]) == 0) {
            return stringToReturn;
        }
        stringToReturn++;
    }
    return 0;
}

static evaluator_kind_t *evaluator_kind_create_internal(
    beater_api_evaluator_kind_TYPE_e type,
    char *pattern,
    double abs,
    double rel,
    long max_micros,
    long max_ms,
    char *model,
    char *rubric,
    char *dom_contains,
    char *url_contains,
    long max_steps,
    double min_ratio
    ) {
    evaluator_kind_t *evaluator_kind_local_var = malloc(sizeof(evaluator_kind_t));
    if (!evaluator_kind_local_var) {
        return NULL;
    }
    evaluator_kind_local_var->type = type;
    evaluator_kind_local_var->pattern = pattern;
    evaluator_kind_local_var->abs = abs;
    evaluator_kind_local_var->rel = rel;
    evaluator_kind_local_var->max_micros = max_micros;
    evaluator_kind_local_var->max_ms = max_ms;
    evaluator_kind_local_var->model = model;
    evaluator_kind_local_var->rubric = rubric;
    evaluator_kind_local_var->dom_contains = dom_contains;
    evaluator_kind_local_var->url_contains = url_contains;
    evaluator_kind_local_var->max_steps = max_steps;
    evaluator_kind_local_var->min_ratio = min_ratio;

    evaluator_kind_local_var->_library_owned = 1;
    return evaluator_kind_local_var;
}

__attribute__((deprecated)) evaluator_kind_t *evaluator_kind_create(
    beater_api_evaluator_kind_TYPE_e type,
    char *pattern,
    double abs,
    double rel,
    long max_micros,
    long max_ms,
    char *model,
    char *rubric,
    char *dom_contains,
    char *url_contains,
    long max_steps,
    double min_ratio
    ) {
    return evaluator_kind_create_internal (
        type,
        pattern,
        abs,
        rel,
        max_micros,
        max_ms,
        model,
        rubric,
        dom_contains,
        url_contains,
        max_steps,
        min_ratio
        );
}

void evaluator_kind_free(evaluator_kind_t *evaluator_kind) {
    if(NULL == evaluator_kind){
        return ;
    }
    if(evaluator_kind->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "evaluator_kind_free");
        return ;
    }
    listEntry_t *listEntry;
    if (evaluator_kind->pattern) {
        free(evaluator_kind->pattern);
        evaluator_kind->pattern = NULL;
    }
    if (evaluator_kind->model) {
        free(evaluator_kind->model);
        evaluator_kind->model = NULL;
    }
    if (evaluator_kind->rubric) {
        free(evaluator_kind->rubric);
        evaluator_kind->rubric = NULL;
    }
    if (evaluator_kind->dom_contains) {
        free(evaluator_kind->dom_contains);
        evaluator_kind->dom_contains = NULL;
    }
    if (evaluator_kind->url_contains) {
        free(evaluator_kind->url_contains);
        evaluator_kind->url_contains = NULL;
    }
    free(evaluator_kind);
}

cJSON *evaluator_kind_convertToJSON(evaluator_kind_t *evaluator_kind) {
    cJSON *item = cJSON_CreateObject();

    // evaluator_kind->type
    if (beater_api_evaluator_kind_TYPE_NULL == evaluator_kind->type) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "type", evaluator_kind_type_ToString(evaluator_kind->type)) == NULL)
    {
    goto fail; //Enum
    }


    // evaluator_kind->pattern
    if (!evaluator_kind->pattern) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "pattern", evaluator_kind->pattern) == NULL) {
    goto fail; //String
    }


    // evaluator_kind->abs
    if (!evaluator_kind->abs) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "abs", evaluator_kind->abs) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind->rel
    if (!evaluator_kind->rel) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "rel", evaluator_kind->rel) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind->max_micros
    if (!evaluator_kind->max_micros) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "max_micros", evaluator_kind->max_micros) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind->max_ms
    if (!evaluator_kind->max_ms) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "max_ms", evaluator_kind->max_ms) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind->model
    if (!evaluator_kind->model) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "model", evaluator_kind->model) == NULL) {
    goto fail; //String
    }


    // evaluator_kind->rubric
    if (!evaluator_kind->rubric) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "rubric", evaluator_kind->rubric) == NULL) {
    goto fail; //String
    }


    // evaluator_kind->dom_contains
    if(evaluator_kind->dom_contains) {
    if(cJSON_AddStringToObject(item, "dom_contains", evaluator_kind->dom_contains) == NULL) {
    goto fail; //String
    }
    }


    // evaluator_kind->url_contains
    if(evaluator_kind->url_contains) {
    if(cJSON_AddStringToObject(item, "url_contains", evaluator_kind->url_contains) == NULL) {
    goto fail; //String
    }
    }


    // evaluator_kind->max_steps
    if (!evaluator_kind->max_steps) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "max_steps", evaluator_kind->max_steps) == NULL) {
    goto fail; //Numeric
    }


    // evaluator_kind->min_ratio
    if (!evaluator_kind->min_ratio) {
        goto fail;
    }
    if(cJSON_AddNumberToObject(item, "min_ratio", evaluator_kind->min_ratio) == NULL) {
    goto fail; //Numeric
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

evaluator_kind_t *evaluator_kind_parseFromJSON(cJSON *evaluator_kindJSON){

    evaluator_kind_t *evaluator_kind_local_var = NULL;

    // evaluator_kind->type
    cJSON *type = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "type");
    if (cJSON_IsNull(type)) {
        type = NULL;
    }
    if (!type) {
        goto end;
    }

    beater_api_evaluator_kind_TYPE_e typeVariable;
    
    if(!cJSON_IsString(type))
    {
    goto end; //Enum
    }
    typeVariable = evaluator_kind_type_FromString(type->valuestring);

    // evaluator_kind->pattern
    cJSON *pattern = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "pattern");
    if (cJSON_IsNull(pattern)) {
        pattern = NULL;
    }
    if (!pattern) {
        goto end;
    }

    
    if(!cJSON_IsString(pattern))
    {
    goto end; //String
    }

    // evaluator_kind->abs
    cJSON *abs = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "abs");
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

    // evaluator_kind->rel
    cJSON *rel = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "rel");
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

    // evaluator_kind->max_micros
    cJSON *max_micros = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "max_micros");
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

    // evaluator_kind->max_ms
    cJSON *max_ms = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "max_ms");
    if (cJSON_IsNull(max_ms)) {
        max_ms = NULL;
    }
    if (!max_ms) {
        goto end;
    }

    
    if(!cJSON_IsNumber(max_ms))
    {
    goto end; //Numeric
    }

    // evaluator_kind->model
    cJSON *model = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "model");
    if (cJSON_IsNull(model)) {
        model = NULL;
    }
    if (!model) {
        goto end;
    }

    
    if(!cJSON_IsString(model))
    {
    goto end; //String
    }

    // evaluator_kind->rubric
    cJSON *rubric = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "rubric");
    if (cJSON_IsNull(rubric)) {
        rubric = NULL;
    }
    if (!rubric) {
        goto end;
    }

    
    if(!cJSON_IsString(rubric))
    {
    goto end; //String
    }

    // evaluator_kind->dom_contains
    cJSON *dom_contains = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "dom_contains");
    if (cJSON_IsNull(dom_contains)) {
        dom_contains = NULL;
    }
    if (dom_contains) { 
    if(!cJSON_IsString(dom_contains) && !cJSON_IsNull(dom_contains))
    {
    goto end; //String
    }
    }

    // evaluator_kind->url_contains
    cJSON *url_contains = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "url_contains");
    if (cJSON_IsNull(url_contains)) {
        url_contains = NULL;
    }
    if (url_contains) { 
    if(!cJSON_IsString(url_contains) && !cJSON_IsNull(url_contains))
    {
    goto end; //String
    }
    }

    // evaluator_kind->max_steps
    cJSON *max_steps = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "max_steps");
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

    // evaluator_kind->min_ratio
    cJSON *min_ratio = cJSON_GetObjectItemCaseSensitive(evaluator_kindJSON, "min_ratio");
    if (cJSON_IsNull(min_ratio)) {
        min_ratio = NULL;
    }
    if (!min_ratio) {
        goto end;
    }

    
    if(!cJSON_IsNumber(min_ratio))
    {
    goto end; //Numeric
    }


    evaluator_kind_local_var = evaluator_kind_create_internal (
        typeVariable,
        strdup(pattern->valuestring),
        abs->valuedouble,
        rel->valuedouble,
        max_micros->valuedouble,
        max_ms->valuedouble,
        strdup(model->valuestring),
        strdup(rubric->valuestring),
        dom_contains && !cJSON_IsNull(dom_contains) ? strdup(dom_contains->valuestring) : NULL,
        url_contains && !cJSON_IsNull(url_contains) ? strdup(url_contains->valuestring) : NULL,
        max_steps->valuedouble,
        min_ratio->valuedouble
        );

    return evaluator_kind_local_var;
end:
    return NULL;

}
