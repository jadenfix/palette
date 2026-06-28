#ifndef evaluator_kind_one_of_9_TEST
#define evaluator_kind_one_of_9_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_9_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_9.h"
evaluator_kind_one_of_9_t* instantiate_evaluator_kind_one_of_9(int include_optional);



evaluator_kind_one_of_9_t* instantiate_evaluator_kind_one_of_9(int include_optional) {
  evaluator_kind_one_of_9_t* evaluator_kind_one_of_9 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_9 = evaluator_kind_one_of_9_create(
      1.337,
      beater_api_evaluator_kind_one_of_9_TYPE_browser_grounding
    );
  } else {
    evaluator_kind_one_of_9 = evaluator_kind_one_of_9_create(
      1.337,
      beater_api_evaluator_kind_one_of_9_TYPE_browser_grounding
    );
  }

  return evaluator_kind_one_of_9;
}


#ifdef evaluator_kind_one_of_9_MAIN

void test_evaluator_kind_one_of_9(int include_optional) {
    evaluator_kind_one_of_9_t* evaluator_kind_one_of_9_1 = instantiate_evaluator_kind_one_of_9(include_optional);

	cJSON* jsonevaluator_kind_one_of_9_1 = evaluator_kind_one_of_9_convertToJSON(evaluator_kind_one_of_9_1);
	printf("evaluator_kind_one_of_9 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_9_1));
	evaluator_kind_one_of_9_t* evaluator_kind_one_of_9_2 = evaluator_kind_one_of_9_parseFromJSON(jsonevaluator_kind_one_of_9_1);
	cJSON* jsonevaluator_kind_one_of_9_2 = evaluator_kind_one_of_9_convertToJSON(evaluator_kind_one_of_9_2);
	printf("repeating evaluator_kind_one_of_9:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_9_2));
}

int main() {
  test_evaluator_kind_one_of_9(1);
  test_evaluator_kind_one_of_9(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_9_MAIN
#endif // evaluator_kind_one_of_9_TEST
