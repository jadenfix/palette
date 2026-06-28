#ifndef evaluator_kind_one_of_5_TEST
#define evaluator_kind_one_of_5_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_5_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_5.h"
evaluator_kind_one_of_5_t* instantiate_evaluator_kind_one_of_5(int include_optional);



evaluator_kind_one_of_5_t* instantiate_evaluator_kind_one_of_5(int include_optional) {
  evaluator_kind_one_of_5_t* evaluator_kind_one_of_5 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_5 = evaluator_kind_one_of_5_create(
      0,
      beater_api_evaluator_kind_one_of_5_TYPE_latency_budget_ms
    );
  } else {
    evaluator_kind_one_of_5 = evaluator_kind_one_of_5_create(
      0,
      beater_api_evaluator_kind_one_of_5_TYPE_latency_budget_ms
    );
  }

  return evaluator_kind_one_of_5;
}


#ifdef evaluator_kind_one_of_5_MAIN

void test_evaluator_kind_one_of_5(int include_optional) {
    evaluator_kind_one_of_5_t* evaluator_kind_one_of_5_1 = instantiate_evaluator_kind_one_of_5(include_optional);

	cJSON* jsonevaluator_kind_one_of_5_1 = evaluator_kind_one_of_5_convertToJSON(evaluator_kind_one_of_5_1);
	printf("evaluator_kind_one_of_5 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_5_1));
	evaluator_kind_one_of_5_t* evaluator_kind_one_of_5_2 = evaluator_kind_one_of_5_parseFromJSON(jsonevaluator_kind_one_of_5_1);
	cJSON* jsonevaluator_kind_one_of_5_2 = evaluator_kind_one_of_5_convertToJSON(evaluator_kind_one_of_5_2);
	printf("repeating evaluator_kind_one_of_5:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_5_2));
}

int main() {
  test_evaluator_kind_one_of_5(1);
  test_evaluator_kind_one_of_5(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_5_MAIN
#endif // evaluator_kind_one_of_5_TEST
