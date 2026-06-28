#ifndef evaluator_kind_one_of_6_TEST
#define evaluator_kind_one_of_6_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_6_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_6.h"
evaluator_kind_one_of_6_t* instantiate_evaluator_kind_one_of_6(int include_optional);



evaluator_kind_one_of_6_t* instantiate_evaluator_kind_one_of_6(int include_optional) {
  evaluator_kind_one_of_6_t* evaluator_kind_one_of_6 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_6 = evaluator_kind_one_of_6_create(
      "0",
      "0",
      beater_api_evaluator_kind_one_of_6_TYPE_llm_judge
    );
  } else {
    evaluator_kind_one_of_6 = evaluator_kind_one_of_6_create(
      "0",
      "0",
      beater_api_evaluator_kind_one_of_6_TYPE_llm_judge
    );
  }

  return evaluator_kind_one_of_6;
}


#ifdef evaluator_kind_one_of_6_MAIN

void test_evaluator_kind_one_of_6(int include_optional) {
    evaluator_kind_one_of_6_t* evaluator_kind_one_of_6_1 = instantiate_evaluator_kind_one_of_6(include_optional);

	cJSON* jsonevaluator_kind_one_of_6_1 = evaluator_kind_one_of_6_convertToJSON(evaluator_kind_one_of_6_1);
	printf("evaluator_kind_one_of_6 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_6_1));
	evaluator_kind_one_of_6_t* evaluator_kind_one_of_6_2 = evaluator_kind_one_of_6_parseFromJSON(jsonevaluator_kind_one_of_6_1);
	cJSON* jsonevaluator_kind_one_of_6_2 = evaluator_kind_one_of_6_convertToJSON(evaluator_kind_one_of_6_2);
	printf("repeating evaluator_kind_one_of_6:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_6_2));
}

int main() {
  test_evaluator_kind_one_of_6(1);
  test_evaluator_kind_one_of_6(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_6_MAIN
#endif // evaluator_kind_one_of_6_TEST
