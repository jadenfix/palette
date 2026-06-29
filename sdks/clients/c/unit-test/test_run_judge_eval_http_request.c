#ifndef run_judge_eval_http_request_TEST
#define run_judge_eval_http_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define run_judge_eval_http_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/run_judge_eval_http_request.h"
run_judge_eval_http_request_t* instantiate_run_judge_eval_http_request(int include_optional);

#include "test_evaluation_case.c"
#include "test_evaluator_spec.c"


run_judge_eval_http_request_t* instantiate_run_judge_eval_http_request(int include_optional) {
  run_judge_eval_http_request_t* run_judge_eval_http_request = NULL;
  if (include_optional) {
    run_judge_eval_http_request = run_judge_eval_http_request_create(
      "0",
       // false, not to have infinite recursion
      instantiate_evaluation_case(0),
       // false, not to have infinite recursion
      instantiate_evaluator_spec(0),
      "0"
    );
  } else {
    run_judge_eval_http_request = run_judge_eval_http_request_create(
      "0",
      NULL,
      NULL,
      "0"
    );
  }

  return run_judge_eval_http_request;
}


#ifdef run_judge_eval_http_request_MAIN

void test_run_judge_eval_http_request(int include_optional) {
    run_judge_eval_http_request_t* run_judge_eval_http_request_1 = instantiate_run_judge_eval_http_request(include_optional);

	cJSON* jsonrun_judge_eval_http_request_1 = run_judge_eval_http_request_convertToJSON(run_judge_eval_http_request_1);
	printf("run_judge_eval_http_request :\n%s\n", cJSON_Print(jsonrun_judge_eval_http_request_1));
	run_judge_eval_http_request_t* run_judge_eval_http_request_2 = run_judge_eval_http_request_parseFromJSON(jsonrun_judge_eval_http_request_1);
	cJSON* jsonrun_judge_eval_http_request_2 = run_judge_eval_http_request_convertToJSON(run_judge_eval_http_request_2);
	printf("repeating run_judge_eval_http_request:\n%s\n", cJSON_Print(jsonrun_judge_eval_http_request_2));
}

int main() {
  test_run_judge_eval_http_request(1);
  test_run_judge_eval_http_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // run_judge_eval_http_request_MAIN
#endif // run_judge_eval_http_request_TEST
