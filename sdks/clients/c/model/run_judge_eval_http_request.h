/*
 * run_judge_eval_http_request.h
 *
 * 
 */

#ifndef _run_judge_eval_http_request_H_
#define _run_judge_eval_http_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct run_judge_eval_http_request_t run_judge_eval_http_request_t;

#include "evaluation_case.h"
#include "evaluator_spec.h"



typedef struct run_judge_eval_http_request_t {
    char *cache_namespace; // string
    struct evaluation_case_t *_case; //model
    struct evaluator_spec_t *evaluator; //model
    char *provider_secret_id; // string

    int _library_owned; // Is the library responsible for freeing this object?
} run_judge_eval_http_request_t;

__attribute__((deprecated)) run_judge_eval_http_request_t *run_judge_eval_http_request_create(
    char *cache_namespace,
    evaluation_case_t *_case,
    evaluator_spec_t *evaluator,
    char *provider_secret_id
);

void run_judge_eval_http_request_free(run_judge_eval_http_request_t *run_judge_eval_http_request);

run_judge_eval_http_request_t *run_judge_eval_http_request_parseFromJSON(cJSON *run_judge_eval_http_requestJSON);

cJSON *run_judge_eval_http_request_convertToJSON(run_judge_eval_http_request_t *run_judge_eval_http_request);

#endif /* _run_judge_eval_http_request_H_ */

