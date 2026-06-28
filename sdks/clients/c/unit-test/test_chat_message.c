#ifndef chat_message_TEST
#define chat_message_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define chat_message_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/chat_message.h"
chat_message_t* instantiate_chat_message(int include_optional);



chat_message_t* instantiate_chat_message(int include_optional) {
  chat_message_t* chat_message = NULL;
  if (include_optional) {
    chat_message = chat_message_create(
      "0",
      "0"
    );
  } else {
    chat_message = chat_message_create(
      "0",
      "0"
    );
  }

  return chat_message;
}


#ifdef chat_message_MAIN

void test_chat_message(int include_optional) {
    chat_message_t* chat_message_1 = instantiate_chat_message(include_optional);

	cJSON* jsonchat_message_1 = chat_message_convertToJSON(chat_message_1);
	printf("chat_message :\n%s\n", cJSON_Print(jsonchat_message_1));
	chat_message_t* chat_message_2 = chat_message_parseFromJSON(jsonchat_message_1);
	cJSON* jsonchat_message_2 = chat_message_convertToJSON(chat_message_2);
	printf("repeating chat_message:\n%s\n", cJSON_Print(jsonchat_message_2));
}

int main() {
  test_chat_message(1);
  test_chat_message(0);

  printf("Hello world \n");
  return 0;
}

#endif // chat_message_MAIN
#endif // chat_message_TEST
