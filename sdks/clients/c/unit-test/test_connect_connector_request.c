#ifndef connect_connector_request_TEST
#define connect_connector_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define connect_connector_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/connect_connector_request.h"
connect_connector_request_t* instantiate_connect_connector_request(int include_optional);



connect_connector_request_t* instantiate_connect_connector_request(int include_optional) {
  connect_connector_request_t* connect_connector_request = NULL;
  if (include_optional) {
    connect_connector_request = connect_connector_request_create(
      "0"
    );
  } else {
    connect_connector_request = connect_connector_request_create(
      "0"
    );
  }

  return connect_connector_request;
}


#ifdef connect_connector_request_MAIN

void test_connect_connector_request(int include_optional) {
    connect_connector_request_t* connect_connector_request_1 = instantiate_connect_connector_request(include_optional);

	cJSON* jsonconnect_connector_request_1 = connect_connector_request_convertToJSON(connect_connector_request_1);
	printf("connect_connector_request :\n%s\n", cJSON_Print(jsonconnect_connector_request_1));
	connect_connector_request_t* connect_connector_request_2 = connect_connector_request_parseFromJSON(jsonconnect_connector_request_1);
	cJSON* jsonconnect_connector_request_2 = connect_connector_request_convertToJSON(connect_connector_request_2);
	printf("repeating connect_connector_request:\n%s\n", cJSON_Print(jsonconnect_connector_request_2));
}

int main() {
  test_connect_connector_request(1);
  test_connect_connector_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // connect_connector_request_MAIN
#endif // connect_connector_request_TEST
