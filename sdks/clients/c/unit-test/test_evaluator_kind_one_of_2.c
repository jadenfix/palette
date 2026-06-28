#ifndef evaluator_kind_one_of_2_TEST
#define evaluator_kind_one_of_2_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_2_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_2.h"
evaluator_kind_one_of_2_t* instantiate_evaluator_kind_one_of_2(int include_optional);



evaluator_kind_one_of_2_t* instantiate_evaluator_kind_one_of_2(int include_optional) {
  evaluator_kind_one_of_2_t* evaluator_kind_one_of_2 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_2 = evaluator_kind_one_of_2_create(
      1.337,
      1.337,
      beater_api_evaluator_kind_one_of_2_TYPE_numeric_tolerance
    );
  } else {
    evaluator_kind_one_of_2 = evaluator_kind_one_of_2_create(
      1.337,
      1.337,
      beater_api_evaluator_kind_one_of_2_TYPE_numeric_tolerance
    );
  }

  return evaluator_kind_one_of_2;
}


#ifdef evaluator_kind_one_of_2_MAIN

void test_evaluator_kind_one_of_2(int include_optional) {
    evaluator_kind_one_of_2_t* evaluator_kind_one_of_2_1 = instantiate_evaluator_kind_one_of_2(include_optional);

	cJSON* jsonevaluator_kind_one_of_2_1 = evaluator_kind_one_of_2_convertToJSON(evaluator_kind_one_of_2_1);
	printf("evaluator_kind_one_of_2 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_2_1));
	evaluator_kind_one_of_2_t* evaluator_kind_one_of_2_2 = evaluator_kind_one_of_2_parseFromJSON(jsonevaluator_kind_one_of_2_1);
	cJSON* jsonevaluator_kind_one_of_2_2 = evaluator_kind_one_of_2_convertToJSON(evaluator_kind_one_of_2_2);
	printf("repeating evaluator_kind_one_of_2:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_2_2));
}

int main() {
  test_evaluator_kind_one_of_2(1);
  test_evaluator_kind_one_of_2(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_2_MAIN
#endif // evaluator_kind_one_of_2_TEST
