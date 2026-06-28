#ifndef chat_completion_response_TEST
#define chat_completion_response_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define chat_completion_response_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/chat_completion_response.h"
chat_completion_response_t* instantiate_chat_completion_response(int include_optional);

#include "test_chat_completion_usage.c"


chat_completion_response_t* instantiate_chat_completion_response(int include_optional) {
  chat_completion_response_t* chat_completion_response = NULL;
  if (include_optional) {
    chat_completion_response = chat_completion_response_create(
      list_createList(),
      56,
      "0",
      "0",
      "0",
       // false, not to have infinite recursion
      instantiate_chat_completion_usage(0)
    );
  } else {
    chat_completion_response = chat_completion_response_create(
      list_createList(),
      56,
      "0",
      "0",
      "0",
      NULL
    );
  }

  return chat_completion_response;
}


#ifdef chat_completion_response_MAIN

void test_chat_completion_response(int include_optional) {
    chat_completion_response_t* chat_completion_response_1 = instantiate_chat_completion_response(include_optional);

	cJSON* jsonchat_completion_response_1 = chat_completion_response_convertToJSON(chat_completion_response_1);
	printf("chat_completion_response :\n%s\n", cJSON_Print(jsonchat_completion_response_1));
	chat_completion_response_t* chat_completion_response_2 = chat_completion_response_parseFromJSON(jsonchat_completion_response_1);
	cJSON* jsonchat_completion_response_2 = chat_completion_response_convertToJSON(chat_completion_response_2);
	printf("repeating chat_completion_response:\n%s\n", cJSON_Print(jsonchat_completion_response_2));
}

int main() {
  test_chat_completion_response(1);
  test_chat_completion_response(0);

  printf("Hello world \n");
  return 0;
}

#endif // chat_completion_response_MAIN
#endif // chat_completion_response_TEST
