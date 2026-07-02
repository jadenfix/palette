#ifndef mine_scenarios_response_TEST
#define mine_scenarios_response_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define mine_scenarios_response_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/mine_scenarios_response.h"
mine_scenarios_response_t* instantiate_mine_scenarios_response(int include_optional);



mine_scenarios_response_t* instantiate_mine_scenarios_response(int include_optional) {
  mine_scenarios_response_t* mine_scenarios_response = NULL;
  if (include_optional) {
    mine_scenarios_response = mine_scenarios_response_create(
      list_createList()
    );
  } else {
    mine_scenarios_response = mine_scenarios_response_create(
      list_createList()
    );
  }

  return mine_scenarios_response;
}


#ifdef mine_scenarios_response_MAIN

void test_mine_scenarios_response(int include_optional) {
    mine_scenarios_response_t* mine_scenarios_response_1 = instantiate_mine_scenarios_response(include_optional);

	cJSON* jsonmine_scenarios_response_1 = mine_scenarios_response_convertToJSON(mine_scenarios_response_1);
	printf("mine_scenarios_response :\n%s\n", cJSON_Print(jsonmine_scenarios_response_1));
	mine_scenarios_response_t* mine_scenarios_response_2 = mine_scenarios_response_parseFromJSON(jsonmine_scenarios_response_1);
	cJSON* jsonmine_scenarios_response_2 = mine_scenarios_response_convertToJSON(mine_scenarios_response_2);
	printf("repeating mine_scenarios_response:\n%s\n", cJSON_Print(jsonmine_scenarios_response_2));
}

int main() {
  test_mine_scenarios_response(1);
  test_mine_scenarios_response(0);

  printf("Hello world \n");
  return 0;
}

#endif // mine_scenarios_response_MAIN
#endif // mine_scenarios_response_TEST
