#ifndef model_routing_TEST
#define model_routing_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define model_routing_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/model_routing.h"
model_routing_t* instantiate_model_routing(int include_optional);

#include "test_model_ref.c"


model_routing_t* instantiate_model_routing(int include_optional) {
  model_routing_t* model_routing = NULL;
  if (include_optional) {
    model_routing = model_routing_create(
      beater_api_model_routing_KIND_managed,
      "0",
       // false, not to have infinite recursion
      instantiate_model_ref(0)
    );
  } else {
    model_routing = model_routing_create(
      beater_api_model_routing_KIND_managed,
      "0",
      NULL
    );
  }

  return model_routing;
}


#ifdef model_routing_MAIN

void test_model_routing(int include_optional) {
    model_routing_t* model_routing_1 = instantiate_model_routing(include_optional);

	cJSON* jsonmodel_routing_1 = model_routing_convertToJSON(model_routing_1);
	printf("model_routing :\n%s\n", cJSON_Print(jsonmodel_routing_1));
	model_routing_t* model_routing_2 = model_routing_parseFromJSON(jsonmodel_routing_1);
	cJSON* jsonmodel_routing_2 = model_routing_convertToJSON(model_routing_2);
	printf("repeating model_routing:\n%s\n", cJSON_Print(jsonmodel_routing_2));
}

int main() {
  test_model_routing(1);
  test_model_routing(0);

  printf("Hello world \n");
  return 0;
}

#endif // model_routing_MAIN
#endif // model_routing_TEST
