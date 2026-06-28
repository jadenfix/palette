#ifndef evaluator_kind_TEST
#define evaluator_kind_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define evaluator_kind_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/evaluator_kind.h"
evaluator_kind_t* instantiate_evaluator_kind(int include_optional);



evaluator_kind_t* instantiate_evaluator_kind(int include_optional) {
  evaluator_kind_t* evaluator_kind = NULL;
  if (include_optional) {
    evaluator_kind = evaluator_kind_create(
      beater_api_evaluator_kind_TYPE_exact_match,
      "0",
      1.337,
      1.337,
      56,
      0,
      "0",
      "0",
      "0",
      "0",
      0,
      1.337
    );
  } else {
    evaluator_kind = evaluator_kind_create(
      beater_api_evaluator_kind_TYPE_exact_match,
      "0",
      1.337,
      1.337,
      56,
      0,
      "0",
      "0",
      "0",
      "0",
      0,
      1.337
    );
  }

  return evaluator_kind;
}


#ifdef evaluator_kind_MAIN

void test_evaluator_kind(int include_optional) {
    evaluator_kind_t* evaluator_kind_1 = instantiate_evaluator_kind(include_optional);

	cJSON* jsonevaluator_kind_1 = evaluator_kind_convertToJSON(evaluator_kind_1);
	printf("evaluator_kind :\n%s\n", cJSON_Print(jsonevaluator_kind_1));
	evaluator_kind_t* evaluator_kind_2 = evaluator_kind_parseFromJSON(jsonevaluator_kind_1);
	cJSON* jsonevaluator_kind_2 = evaluator_kind_convertToJSON(evaluator_kind_2);
	printf("repeating evaluator_kind:\n%s\n", cJSON_Print(jsonevaluator_kind_2));
}

int main() {
  test_evaluator_kind(1);
  test_evaluator_kind(0);

  printf("Hello world \n");
  return 0;
}

#endif // evaluator_kind_MAIN
#endif // evaluator_kind_TEST
