#ifndef invoke_connector_request_TEST
#define invoke_connector_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define invoke_connector_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/invoke_connector_request.h"
invoke_connector_request_t* instantiate_invoke_connector_request(int include_optional);



invoke_connector_request_t* instantiate_invoke_connector_request(int include_optional) {
  invoke_connector_request_t* invoke_connector_request = NULL;
  if (include_optional) {
    invoke_connector_request = invoke_connector_request_create(
      0,
      "0"
    );
  } else {
    invoke_connector_request = invoke_connector_request_create(
      0,
      "0"
    );
  }

  return invoke_connector_request;
}


#ifdef invoke_connector_request_MAIN

void test_invoke_connector_request(int include_optional) {
    invoke_connector_request_t* invoke_connector_request_1 = instantiate_invoke_connector_request(include_optional);

	cJSON* jsoninvoke_connector_request_1 = invoke_connector_request_convertToJSON(invoke_connector_request_1);
	printf("invoke_connector_request :\n%s\n", cJSON_Print(jsoninvoke_connector_request_1));
	invoke_connector_request_t* invoke_connector_request_2 = invoke_connector_request_parseFromJSON(jsoninvoke_connector_request_1);
	cJSON* jsoninvoke_connector_request_2 = invoke_connector_request_convertToJSON(invoke_connector_request_2);
	printf("repeating invoke_connector_request:\n%s\n", cJSON_Print(jsoninvoke_connector_request_2));
}

int main() {
  test_invoke_connector_request(1);
  test_invoke_connector_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // invoke_connector_request_MAIN
#endif // invoke_connector_request_TEST
