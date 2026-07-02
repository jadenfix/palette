#ifndef list_scenarios_response_TEST
#define list_scenarios_response_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define list_scenarios_response_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/list_scenarios_response.h"
list_scenarios_response_t* instantiate_list_scenarios_response(int include_optional);



list_scenarios_response_t* instantiate_list_scenarios_response(int include_optional) {
  list_scenarios_response_t* list_scenarios_response = NULL;
  if (include_optional) {
    list_scenarios_response = list_scenarios_response_create(
      "0",
      list_createList()
    );
  } else {
    list_scenarios_response = list_scenarios_response_create(
      "0",
      list_createList()
    );
  }

  return list_scenarios_response;
}


#ifdef list_scenarios_response_MAIN

void test_list_scenarios_response(int include_optional) {
    list_scenarios_response_t* list_scenarios_response_1 = instantiate_list_scenarios_response(include_optional);

	cJSON* jsonlist_scenarios_response_1 = list_scenarios_response_convertToJSON(list_scenarios_response_1);
	printf("list_scenarios_response :\n%s\n", cJSON_Print(jsonlist_scenarios_response_1));
	list_scenarios_response_t* list_scenarios_response_2 = list_scenarios_response_parseFromJSON(jsonlist_scenarios_response_1);
	cJSON* jsonlist_scenarios_response_2 = list_scenarios_response_convertToJSON(list_scenarios_response_2);
	printf("repeating list_scenarios_response:\n%s\n", cJSON_Print(jsonlist_scenarios_response_2));
}

int main() {
  test_list_scenarios_response(1);
  test_list_scenarios_response(0);

  printf("Hello world \n");
  return 0;
}

#endif // list_scenarios_response_MAIN
#endif // list_scenarios_response_TEST
