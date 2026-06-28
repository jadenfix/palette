#ifndef model_routing_one_of_1_TEST
#define model_routing_one_of_1_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define model_routing_one_of_1_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/model_routing_one_of_1.h"
model_routing_one_of_1_t* instantiate_model_routing_one_of_1(int include_optional);

#include "test_model_ref.c"


model_routing_one_of_1_t* instantiate_model_routing_one_of_1(int include_optional) {
  model_routing_one_of_1_t* model_routing_one_of_1 = NULL;
  if (include_optional) {
    model_routing_one_of_1 = model_routing_one_of_1_create(
       // false, not to have infinite recursion
      instantiate_model_ref(0),
      beater_api_model_routing_one_of_1_KIND_managed
    );
  } else {
    model_routing_one_of_1 = model_routing_one_of_1_create(
      NULL,
      beater_api_model_routing_one_of_1_KIND_managed
    );
  }

  return model_routing_one_of_1;
}


#ifdef model_routing_one_of_1_MAIN

void test_model_routing_one_of_1(int include_optional) {
    model_routing_one_of_1_t* model_routing_one_of_1_1 = instantiate_model_routing_one_of_1(include_optional);

	cJSON* jsonmodel_routing_one_of_1_1 = model_routing_one_of_1_convertToJSON(model_routing_one_of_1_1);
	printf("model_routing_one_of_1 :\n%s\n", cJSON_Print(jsonmodel_routing_one_of_1_1));
	model_routing_one_of_1_t* model_routing_one_of_1_2 = model_routing_one_of_1_parseFromJSON(jsonmodel_routing_one_of_1_1);
	cJSON* jsonmodel_routing_one_of_1_2 = model_routing_one_of_1_convertToJSON(model_routing_one_of_1_2);
	printf("repeating model_routing_one_of_1:\n%s\n", cJSON_Print(jsonmodel_routing_one_of_1_2));
}

int main() {
  test_model_routing_one_of_1(1);
  test_model_routing_one_of_1(0);

  printf("Hello world \n");
  return 0;
}

#endif // model_routing_one_of_1_MAIN
#endif // model_routing_one_of_1_TEST
