#ifndef evaluator_kind_one_of_3_TEST
#define evaluator_kind_one_of_3_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_one_of_3_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind_one_of_3.h"
evaluator_kind_one_of_3_t* instantiate_evaluator_kind_one_of_3(int include_optional);



evaluator_kind_one_of_3_t* instantiate_evaluator_kind_one_of_3(int include_optional) {
  evaluator_kind_one_of_3_t* evaluator_kind_one_of_3 = NULL;
  if (include_optional) {
    evaluator_kind_one_of_3 = evaluator_kind_one_of_3_create(
      beater_api_evaluator_kind_one_of_3_TYPE_json_object
    );
  } else {
    evaluator_kind_one_of_3 = evaluator_kind_one_of_3_create(
      beater_api_evaluator_kind_one_of_3_TYPE_json_object
    );
  }

  return evaluator_kind_one_of_3;
}


#ifdef evaluator_kind_one_of_3_MAIN

void test_evaluator_kind_one_of_3(int include_optional) {
    evaluator_kind_one_of_3_t* evaluator_kind_one_of_3_1 = instantiate_evaluator_kind_one_of_3(include_optional);

	cJSON* jsonevaluator_kind_one_of_3_1 = evaluator_kind_one_of_3_convertToJSON(evaluator_kind_one_of_3_1);
	printf("evaluator_kind_one_of_3 :\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_3_1));
	evaluator_kind_one_of_3_t* evaluator_kind_one_of_3_2 = evaluator_kind_one_of_3_parseFromJSON(jsonevaluator_kind_one_of_3_1);
	cJSON* jsonevaluator_kind_one_of_3_2 = evaluator_kind_one_of_3_convertToJSON(evaluator_kind_one_of_3_2);
	printf("repeating evaluator_kind_one_of_3:\n%s\n", cJSON_Print(jsonevaluator_kind_one_of_3_2));
}

int main() {
  test_evaluator_kind_one_of_3(1);
  test_evaluator_kind_one_of_3(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_one_of_3_MAIN
#endif // evaluator_kind_one_of_3_TEST
