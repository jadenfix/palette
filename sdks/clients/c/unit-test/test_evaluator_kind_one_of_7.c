#ifndef evaluator_kind_one_of_7_TEST
#define evaluator_kind_one_of_7_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_7_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_7.h"
evaluator_kind_one_of_7_t* instantiate_evaluator_kind_one_of_7(int include_optional);



evaluator_kind_one_of_7_t* instantiate_evaluator_kind_one_of_7(int include_optional) {
  evaluator_kind_one_of_7_t* evaluator_kind_one_of_7 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_7 = evaluator_kind_one_of_7_create(
      "0",
      beater_api_evaluator_kind_one_of_7_TYPE_browser_task_success,
      "0"
    );
  } else {
    evaluator_kind_one_of_7 = evaluator_kind_one_of_7_create(
      "0",
      beater_api_evaluator_kind_one_of_7_TYPE_browser_task_success,
      "0"
    );
  }

  return evaluator_kind_one_of_7;
}


#ifdef evaluator_kind_one_of_7_MAIN

void test_evaluator_kind_one_of_7(int include_optional) {
    evaluator_kind_one_of_7_t* evaluator_kind_one_of_7_1 = instantiate_evaluator_kind_one_of_7(include_optional);

	cJSON* jsonevaluator_kind_one_of_7_1 = evaluator_kind_one_of_7_convertToJSON(evaluator_kind_one_of_7_1);
	printf("evaluator_kind_one_of_7 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_7_1));
	evaluator_kind_one_of_7_t* evaluator_kind_one_of_7_2 = evaluator_kind_one_of_7_parseFromJSON(jsonevaluator_kind_one_of_7_1);
	cJSON* jsonevaluator_kind_one_of_7_2 = evaluator_kind_one_of_7_convertToJSON(evaluator_kind_one_of_7_2);
	printf("repeating evaluator_kind_one_of_7:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_7_2));
}

int main() {
  test_evaluator_kind_one_of_7(1);
  test_evaluator_kind_one_of_7(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_7_MAIN
#endif // evaluator_kind_one_of_7_TEST
