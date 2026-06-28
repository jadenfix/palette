#ifndef gateway_chat_http_request_TEST
#define gateway_chat_http_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define gateway_chat_http_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/gateway_chat_http_request.h"
gateway_chat_http_request_t* instantiate_gateway_chat_http_request(int include_optional);

#include "test_chat_completion_request.c"


gateway_chat_http_request_t* instantiate_gateway_chat_http_request(int include_optional) {
  gateway_chat_http_request_t* gateway_chat_http_request = NULL;
  if (include_optional) {
    gateway_chat_http_request = gateway_chat_http_request_create(
      56,
       // false, not to have infinite recursion
      instantiate_chat_completion_request(0),
      list_createList()
    );
  } else {
    gateway_chat_http_request = gateway_chat_http_request_create(
      56,
      NULL,
      list_createList()
    );
  }

  return gateway_chat_http_request;
}


#ifdef gateway_chat_http_request_MAIN

void test_gateway_chat_http_request(int include_optional) {
    gateway_chat_http_request_t* gateway_chat_http_request_1 = instantiate_gateway_chat_http_request(include_optional);

	cJSON* jsongateway_chat_http_request_1 = gateway_chat_http_request_convertToJSON(gateway_chat_http_request_1);
	printf("gateway_chat_http_request :\n%s\n", cJSON_Print(jsongateway_chat_http_request_1));
	gateway_chat_http_request_t* gateway_chat_http_request_2 = gateway_chat_http_request_parseFromJSON(jsongateway_chat_http_request_1);
	cJSON* jsongateway_chat_http_request_2 = gateway_chat_http_request_convertToJSON(gateway_chat_http_request_2);
	printf("repeating gateway_chat_http_request:\n%s\n", cJSON_Print(jsongateway_chat_http_request_2));
}

int main() {
  test_gateway_chat_http_request(1);
  test_gateway_chat_http_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // gateway_chat_http_request_MAIN
#endif // gateway_chat_http_request_TEST
