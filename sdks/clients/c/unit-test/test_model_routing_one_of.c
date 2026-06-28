#ifndef model_routing_one_of_TEST
#define model_routing_one_of_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define model_routing_one_of_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/model_routing_one_of.h"
model_routing_one_of_t* instantiate_model_routing_one_of(int include_optional);



model_routing_one_of_t* instantiate_model_routing_one_of(int include_optional) {
  model_routing_one_of_t* model_routing_one_of = NULL;
  if (include_optional) {
    model_routing_one_of = model_routing_one_of_create(
      beater_api_model_routing_one_of_KIND_byok,
      "0"
    );
  } else {
    model_routing_one_of = model_routing_one_of_create(
      beater_api_model_routing_one_of_KIND_byok,
      "0"
    );
  }

  return model_routing_one_of;
}


#ifdef model_routing_one_of_MAIN

void test_model_routing_one_of(int include_optional) {
    model_routing_one_of_t* model_routing_one_of_1 = instantiate_model_routing_one_of(include_optional);

	cJSON* jsonmodel_routing_one_of_1 = model_routing_one_of_convertToJSON(model_routing_one_of_1);
	printf("model_routing_one_of :\n%s\n", cJSON_Print(jsonmodel_routing_one_of_1));
	model_routing_one_of_t* model_routing_one_of_2 = model_routing_one_of_parseFromJSON(jsonmodel_routing_one_of_1);
	cJSON* jsonmodel_routing_one_of_2 = model_routing_one_of_convertToJSON(model_routing_one_of_2);
	printf("repeating model_routing_one_of:\n%s\n", cJSON_Print(jsonmodel_routing_one_of_2));
}

int main() {
  test_model_routing_one_of(1);
  test_model_routing_one_of(0);

  printf("Hello world \n");
  return 0;
}

#endif // model_routing_one_of_MAIN
#endif // model_routing_one_of_TEST
