#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "run_judge_eval_http_request.h"



static run_judge_eval_http_request_t *run_judge_eval_http_request_create_internal(
    char *cache_namespace,
    evaluation_case_t *_case,
    evaluator_spec_t *evaluator,
    char *provider_secret_id
    ) {
    run_judge_eval_http_request_t *run_judge_eval_http_request_local_var = malloc(sizeof(run_judge_eval_http_request_t));
    if (!run_judge_eval_http_request_local_var) {
        return NULL;
    }
    run_judge_eval_http_request_local_var->cache_namespace = cache_namespace;
    run_judge_eval_http_request_local_var->_case = _case;
    run_judge_eval_http_request_local_var->evaluator = evaluator;
    run_judge_eval_http_request_local_var->provider_secret_id = provider_secret_id;

    run_judge_eval_http_request_local_var->_library_owned = 1;
    return run_judge_eval_http_request_local_var;
}

__attribute__((deprecated)) run_judge_eval_http_request_t *run_judge_eval_http_request_create(
    char *cache_namespace,
    evaluation_case_t *_case,
    evaluator_spec_t *evaluator,
    char *provider_secret_id
    ) {
    return run_judge_eval_http_request_create_internal (
        cache_namespace,
        _case,
        evaluator,
        provider_secret_id
        );
}

void run_judge_eval_http_request_free(run_judge_eval_http_request_t *run_judge_eval_http_request) {
    if(NULL == run_judge_eval_http_request){
        return ;
    }
    if(run_judge_eval_http_request->_library_owned != 1){
        fprintf(stderr, "WARNING: %s() does NOT free objects allocated by the user\n", "run_judge_eval_http_request_free");
        return ;
    }
    listEntry_t *listEntry;
    if (run_judge_eval_http_request->cache_namespace) {
        free(run_judge_eval_http_request->cache_namespace);
        run_judge_eval_http_request->cache_namespace = NULL;
    }
    if (run_judge_eval_http_request->_case) {
        evaluation_case_free(run_judge_eval_http_request->_case);
        run_judge_eval_http_request->_case = NULL;
    }
    if (run_judge_eval_http_request->evaluator) {
        evaluator_spec_free(run_judge_eval_http_request->evaluator);
        run_judge_eval_http_request->evaluator = NULL;
    }
    if (run_judge_eval_http_request->provider_secret_id) {
        free(run_judge_eval_http_request->provider_secret_id);
        run_judge_eval_http_request->provider_secret_id = NULL;
    }
    free(run_judge_eval_http_request);
}

cJSON *run_judge_eval_http_request_convertToJSON(run_judge_eval_http_request_t *run_judge_eval_http_request) {
    cJSON *item = cJSON_CreateObject();

    // run_judge_eval_http_request->cache_namespace
    if(run_judge_eval_http_request->cache_namespace) {
    if(cJSON_AddStringToObject(item, "cache_namespace", run_judge_eval_http_request->cache_namespace) == NULL) {
    goto fail; //String
    }
    }


    // run_judge_eval_http_request->_case
    if (!run_judge_eval_http_request->_case) {
        goto fail;
    }
    cJSON *_case_local_JSON = evaluation_case_convertToJSON(run_judge_eval_http_request->_case);
    if(_case_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "case", _case_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // run_judge_eval_http_request->evaluator
    if (!run_judge_eval_http_request->evaluator) {
        goto fail;
    }
    cJSON *evaluator_local_JSON = evaluator_spec_convertToJSON(run_judge_eval_http_request->evaluator);
    if(evaluator_local_JSON == NULL) {
    goto fail; //model
    }
    cJSON_AddItemToObject(item, "evaluator", evaluator_local_JSON);
    if(item->child == NULL) {
    goto fail;
    }


    // run_judge_eval_http_request->provider_secret_id
    if (!run_judge_eval_http_request->provider_secret_id) {
        goto fail;
    }
    if(cJSON_AddStringToObject(item, "provider_secret_id", run_judge_eval_http_request->provider_secret_id) == NULL) {
    goto fail; //String
    }

    return item;
fail:
    if (item) {
        cJSON_Delete(item);
    }
    return NULL;
}

run_judge_eval_http_request_t *run_judge_eval_http_request_parseFromJSON(cJSON *run_judge_eval_http_requestJSON){

    run_judge_eval_http_request_t *run_judge_eval_http_request_local_var = NULL;

    // define the local variable for run_judge_eval_http_request->_case
    evaluation_case_t *_case_local_nonprim = NULL;

    // define the local variable for run_judge_eval_http_request->evaluator
    evaluator_spec_t *evaluator_local_nonprim = NULL;

    // run_judge_eval_http_request->cache_namespace
    cJSON *cache_namespace = cJSON_GetObjectItemCaseSensitive(run_judge_eval_http_requestJSON, "cache_namespace");
    if (cJSON_IsNull(cache_namespace)) {
        cache_namespace = NULL;
    }
    if (cache_namespace) { 
    if(!cJSON_IsString(cache_namespace) && !cJSON_IsNull(cache_namespace))
    {
    goto end; //String
    }
    }

    // run_judge_eval_http_request->_case
    cJSON *_case = cJSON_GetObjectItemCaseSensitive(run_judge_eval_http_requestJSON, "case");
    if (cJSON_IsNull(_case)) {
        _case = NULL;
    }
    if (!_case) {
        goto end;
    }

    
    _case_local_nonprim = evaluation_case_parseFromJSON(_case); //nonprimitive

    // run_judge_eval_http_request->evaluator
    cJSON *evaluator = cJSON_GetObjectItemCaseSensitive(run_judge_eval_http_requestJSON, "evaluator");
    if (cJSON_IsNull(evaluator)) {
        evaluator = NULL;
    }
    if (!evaluator) {
        goto end;
    }

    
    evaluator_local_nonprim = evaluator_spec_parseFromJSON(evaluator); //nonprimitive

    // run_judge_eval_http_request->provider_secret_id
    cJSON *provider_secret_id = cJSON_GetObjectItemCaseSensitive(run_judge_eval_http_requestJSON, "provider_secret_id");
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


    run_judge_eval_http_request_local_var = run_judge_eval_http_request_create_internal (
        cache_namespace && !cJSON_IsNull(cache_namespace) ? strdup(cache_namespace->valuestring) : NULL,
        _case_local_nonprim,
        evaluator_local_nonprim,
        strdup(provider_secret_id->valuestring)
        );

    return run_judge_eval_http_request_local_var;
end:
    if (_case_local_nonprim) {
        evaluation_case_free(_case_local_nonprim);
        _case_local_nonprim = NULL;
    }
    if (evaluator_local_nonprim) {
        evaluator_spec_free(evaluator_local_nonprim);
        evaluator_local_nonprim = NULL;
    }
    return NULL;

}
