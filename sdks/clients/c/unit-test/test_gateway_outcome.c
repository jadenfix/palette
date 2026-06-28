#ifndef gateway_outcome_TEST
#define gateway_outcome_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define gateway_outcome_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/gateway_outcome.h"
gateway_outcome_t* instantiate_gateway_outcome(int include_optional);

#include "test_money.c"
#include "test_model_ref.c"
#include "test_chat_completion_response.c"
#include "test_token_counts.c"


gateway_outcome_t* instantiate_gateway_outcome(int include_optional) {
  gateway_outcome_t* gateway_outcome = NULL;
  if (include_optional) {
    gateway_outcome = gateway_outcome_create(
      0,
      1,
       // false, not to have infinite recursion
      instantiate_money(0),
       // false, not to have infinite recursion
      instantiate_model_ref(0),
      "0",
      "0",
       // false, not to have infinite recursion
      instantiate_chat_completion_response(0),
       // false, not to have infinite recursion
      instantiate_token_counts(0)
    );
  } else {
    gateway_outcome = gateway_outcome_create(
      0,
      1,
      NULL,
      NULL,
      "0",
      "0",
      NULL,
      NULL
    );
  }

  return gateway_outcome;
}


#ifdef gateway_outcome_MAIN

void test_gateway_outcome(int include_optional) {
    gateway_outcome_t* gateway_outcome_1 = instantiate_gateway_outcome(include_optional);

	cJSON* jsongateway_outcome_1 = gateway_outcome_convertToJSON(gateway_outcome_1);
	printf("gateway_outcome :\n%s\n", cJSON_Print(jsongateway_outcome_1));
	gateway_outcome_t* gateway_outcome_2 = gateway_outcome_parseFromJSON(jsongateway_outcome_1);
	cJSON* jsongateway_outcome_2 = gateway_outcome_convertToJSON(gateway_outcome_2);
	printf("repeating gateway_outcome:\n%s\n", cJSON_Print(jsongateway_outcome_2));
}

int main() {
  test_gateway_outcome(1);
  test_gateway_outcome(0);

  printf("Hello world \n");
  return 0;
}

#endif // gateway_outcome_MAIN
#endif // gateway_outcome_TEST
