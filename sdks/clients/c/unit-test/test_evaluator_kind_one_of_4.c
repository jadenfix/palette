#ifndef evaluator_kind_one_of_4_TEST
#define evaluator_kind_one_of_4_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_4_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_4.h"
evaluator_kind_one_of_4_t* instantiate_evaluator_kind_one_of_4(int include_optional);



evaluator_kind_one_of_4_t* instantiate_evaluator_kind_one_of_4(int include_optional) {
  evaluator_kind_one_of_4_t* evaluator_kind_one_of_4 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_4 = evaluator_kind_one_of_4_create(
      56,
      beater_api_evaluator_kind_one_of_4_TYPE_cost_budget
    );
  } else {
    evaluator_kind_one_of_4 = evaluator_kind_one_of_4_create(
      56,
      beater_api_evaluator_kind_one_of_4_TYPE_cost_budget
    );
  }

  return evaluator_kind_one_of_4;
}


#ifdef evaluator_kind_one_of_4_MAIN

void test_evaluator_kind_one_of_4(int include_optional) {
    evaluator_kind_one_of_4_t* evaluator_kind_one_of_4_1 = instantiate_evaluator_kind_one_of_4(include_optional);

	cJSON* jsonevaluator_kind_one_of_4_1 = evaluator_kind_one_of_4_convertToJSON(evaluator_kind_one_of_4_1);
	printf("evaluator_kind_one_of_4 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_4_1));
	evaluator_kind_one_of_4_t* evaluator_kind_one_of_4_2 = evaluator_kind_one_of_4_parseFromJSON(jsonevaluator_kind_one_of_4_1);
	cJSON* jsonevaluator_kind_one_of_4_2 = evaluator_kind_one_of_4_convertToJSON(evaluator_kind_one_of_4_2);
	printf("repeating evaluator_kind_one_of_4:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_4_2));
}

int main() {
  test_evaluator_kind_one_of_4(1);
  test_evaluator_kind_one_of_4(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_4_MAIN
#endif // evaluator_kind_one_of_4_TEST
