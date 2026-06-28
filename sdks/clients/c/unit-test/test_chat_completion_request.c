#ifndef chat_completion_request_TEST
#define chat_completion_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define chat_completion_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/chat_completion_request.h"
chat_completion_request_t* instantiate_chat_completion_request(int include_optional);



chat_completion_request_t* instantiate_chat_completion_request(int include_optional) {
  chat_completion_request_t* chat_completion_request = NULL;
  if (include_optional) {
    chat_completion_request = chat_completion_request_create(
      0,
      list_createList(),
      "0",
      "0",
      1.337,
      1.337
    );
  } else {
    chat_completion_request = chat_completion_request_create(
      0,
      list_createList(),
      "0",
      "0",
      1.337,
      1.337
    );
  }

  return chat_completion_request;
}


#ifdef chat_completion_request_MAIN

void test_chat_completion_request(int include_optional) {
    chat_completion_request_t* chat_completion_request_1 = instantiate_chat_completion_request(include_optional);

	cJSON* jsonchat_completion_request_1 = chat_completion_request_convertToJSON(chat_completion_request_1);
	printf("chat_completion_request :\n%s\n", cJSON_Print(jsonchat_completion_request_1));
	chat_completion_request_t* chat_completion_request_2 = chat_completion_request_parseFromJSON(jsonchat_completion_request_1);
	cJSON* jsonchat_completion_request_2 = chat_completion_request_convertToJSON(chat_completion_request_2);
	printf("repeating chat_completion_request:\n%s\n", cJSON_Print(jsonchat_completion_request_2));
}

int main() {
  test_chat_completion_request(1);
  test_chat_completion_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // chat_completion_request_MAIN
#endif // chat_completion_request_TEST
