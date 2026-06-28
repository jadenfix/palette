#ifndef evaluator_kind_one_of_8_TEST
#define evaluator_kind_one_of_8_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_8_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_8.h"
evaluator_kind_one_of_8_t* instantiate_evaluator_kind_one_of_8(int include_optional);



evaluator_kind_one_of_8_t* instantiate_evaluator_kind_one_of_8(int include_optional) {
  evaluator_kind_one_of_8_t* evaluator_kind_one_of_8 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_8 = evaluator_kind_one_of_8_create(
      0,
      beater_api_evaluator_kind_one_of_8_TYPE_browser_step_efficiency
    );
  } else {
    evaluator_kind_one_of_8 = evaluator_kind_one_of_8_create(
      0,
      beater_api_evaluator_kind_one_of_8_TYPE_browser_step_efficiency
    );
  }

  return evaluator_kind_one_of_8;
}


#ifdef evaluator_kind_one_of_8_MAIN

void test_evaluator_kind_one_of_8(int include_optional) {
    evaluator_kind_one_of_8_t* evaluator_kind_one_of_8_1 = instantiate_evaluator_kind_one_of_8(include_optional);

	cJSON* jsonevaluator_kind_one_of_8_1 = evaluator_kind_one_of_8_convertToJSON(evaluator_kind_one_of_8_1);
	printf("evaluator_kind_one_of_8 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_8_1));
	evaluator_kind_one_of_8_t* evaluator_kind_one_of_8_2 = evaluator_kind_one_of_8_parseFromJSON(jsonevaluator_kind_one_of_8_1);
	cJSON* jsonevaluator_kind_one_of_8_2 = evaluator_kind_one_of_8_convertToJSON(evaluator_kind_one_of_8_2);
	printf("repeating evaluator_kind_one_of_8:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_8_2));
}

int main() {
  test_evaluator_kind_one_of_8(1);
  test_evaluator_kind_one_of_8(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_8_MAIN
#endif // evaluator_kind_one_of_8_TEST
