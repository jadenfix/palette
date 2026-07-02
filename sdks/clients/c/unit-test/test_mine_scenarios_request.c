#ifndef mine_scenarios_request_TEST
#define mine_scenarios_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define mine_scenarios_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/mine_scenarios_request.h"
mine_scenarios_request_t* instantiate_mine_scenarios_request(int include_optional);



mine_scenarios_request_t* instantiate_mine_scenarios_request(int include_optional) {
  mine_scenarios_request_t* mine_scenarios_request = NULL;
  if (include_optional) {
    mine_scenarios_request = mine_scenarios_request_create(
      1.337,
      list_createList()
    );
  } else {
    mine_scenarios_request = mine_scenarios_request_create(
      1.337,
      list_createList()
    );
  }

  return mine_scenarios_request;
}


#ifdef mine_scenarios_request_MAIN

void test_mine_scenarios_request(int include_optional) {
    mine_scenarios_request_t* mine_scenarios_request_1 = instantiate_mine_scenarios_request(include_optional);

	cJSON* jsonmine_scenarios_request_1 = mine_scenarios_request_convertToJSON(mine_scenarios_request_1);
	printf("mine_scenarios_request :\n%s\n", cJSON_Print(jsonmine_scenarios_request_1));
	mine_scenarios_request_t* mine_scenarios_request_2 = mine_scenarios_request_parseFromJSON(jsonmine_scenarios_request_1);
	cJSON* jsonmine_scenarios_request_2 = mine_scenarios_request_convertToJSON(mine_scenarios_request_2);
	printf("repeating mine_scenarios_request:\n%s\n", cJSON_Print(jsonmine_scenarios_request_2));
}

int main() {
  test_mine_scenarios_request(1);
  test_mine_scenarios_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // mine_scenarios_request_MAIN
#endif // mine_scenarios_request_TEST
