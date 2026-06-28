#ifndef chat_completion_usage_TEST
#define chat_completion_usage_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define chat_completion_usage_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/chat_completion_usage.h"
chat_completion_usage_t* instantiate_chat_completion_usage(int include_optional);



chat_completion_usage_t* instantiate_chat_completion_usage(int include_optional) {
  chat_completion_usage_t* chat_completion_usage = NULL;
  if (include_optional) {
    chat_completion_usage = chat_completion_usage_create(
      0,
      0,
      0
    );
  } else {
    chat_completion_usage = chat_completion_usage_create(
      0,
      0,
      0
    );
  }

  return chat_completion_usage;
}


#ifdef chat_completion_usage_MAIN

void test_chat_completion_usage(int include_optional) {
    chat_completion_usage_t* chat_completion_usage_1 = instantiate_chat_completion_usage(include_optional);

	cJSON* jsonchat_completion_usage_1 = chat_completion_usage_convertToJSON(chat_completion_usage_1);
	printf("chat_completion_usage :\n%s\n", cJSON_Print(jsonchat_completion_usage_1));
	chat_completion_usage_t* chat_completion_usage_2 = chat_completion_usage_parseFromJSON(jsonchat_completion_usage_1);
	cJSON* jsonchat_completion_usage_2 = chat_completion_usage_convertToJSON(chat_completion_usage_2);
	printf("repeating chat_completion_usage:\n%s\n", cJSON_Print(jsonchat_completion_usage_2));
}

int main() {
  test_chat_completion_usage(1);
  test_chat_completion_usage(0);

  printf("Hello world \n");
  return 0;
}

#endif // chat_completion_usage_MAIN
#endif // chat_completion_usage_TEST
