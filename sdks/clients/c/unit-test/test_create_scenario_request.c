#ifndef create_scenario_request_TEST
#define create_scenario_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define create_scenario_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/create_scenario_request.h"
create_scenario_request_t* instantiate_create_scenario_request(int include_optional);



create_scenario_request_t* instantiate_create_scenario_request(int include_optional) {
  create_scenario_request_t* create_scenario_request = NULL;
  if (include_optional) {
    create_scenario_request = create_scenario_request_create(
      "0",
      "0",
      beater_api_create_scenario_request__tool_error,
      list_createList(),
      "0"
    );
  } else {
    create_scenario_request = create_scenario_request_create(
      "0",
      "0",
      beater_api_create_scenario_request__tool_error,
      list_createList(),
      "0"
    );
  }

  return create_scenario_request;
}


#ifdef create_scenario_request_MAIN

void test_create_scenario_request(int include_optional) {
    create_scenario_request_t* create_scenario_request_1 = instantiate_create_scenario_request(include_optional);

	cJSON* jsoncreate_scenario_request_1 = create_scenario_request_convertToJSON(create_scenario_request_1);
	printf("create_scenario_request :\n%s\n", cJSON_Print(jsoncreate_scenario_request_1));
	create_scenario_request_t* create_scenario_request_2 = create_scenario_request_parseFromJSON(jsoncreate_scenario_request_1);
	cJSON* jsoncreate_scenario_request_2 = create_scenario_request_convertToJSON(create_scenario_request_2);
	printf("repeating create_scenario_request:\n%s\n", cJSON_Print(jsoncreate_scenario_request_2));
}

int main() {
  test_create_scenario_request(1);
  test_create_scenario_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // create_scenario_request_MAIN
#endif // create_scenario_request_TEST
