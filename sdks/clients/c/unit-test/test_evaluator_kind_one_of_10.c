#ifndef evaluator_kind_one_of_10_TEST
#define evaluator_kind_one_of_10_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_10_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_10.h"
evaluator_kind_one_of_10_t* instantiate_evaluator_kind_one_of_10(int include_optional);



evaluator_kind_one_of_10_t* instantiate_evaluator_kind_one_of_10(int include_optional) {
  evaluator_kind_one_of_10_t* evaluator_kind_one_of_10 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_10 = evaluator_kind_one_of_10_create(
      beater_api_evaluator_kind_one_of_10_TYPE_browser_recovery
    );
  } else {
    evaluator_kind_one_of_10 = evaluator_kind_one_of_10_create(
      beater_api_evaluator_kind_one_of_10_TYPE_browser_recovery
    );
  }

  return evaluator_kind_one_of_10;
}


#ifdef evaluator_kind_one_of_10_MAIN

void test_evaluator_kind_one_of_10(int include_optional) {
    evaluator_kind_one_of_10_t* evaluator_kind_one_of_10_1 = instantiate_evaluator_kind_one_of_10(include_optional);

	cJSON* jsonevaluator_kind_one_of_10_1 = evaluator_kind_one_of_10_convertToJSON(evaluator_kind_one_of_10_1);
	printf("evaluator_kind_one_of_10 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_10_1));
	evaluator_kind_one_of_10_t* evaluator_kind_one_of_10_2 = evaluator_kind_one_of_10_parseFromJSON(jsonevaluator_kind_one_of_10_1);
	cJSON* jsonevaluator_kind_one_of_10_2 = evaluator_kind_one_of_10_convertToJSON(evaluator_kind_one_of_10_2);
	printf("repeating evaluator_kind_one_of_10:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_10_2));
}

int main() {
  test_evaluator_kind_one_of_10(1);
  test_evaluator_kind_one_of_10(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_10_MAIN
#endif // evaluator_kind_one_of_10_TEST
