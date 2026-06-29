#ifndef prompt_list_response_TEST
#define prompt_list_response_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_list_response_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt_list_response.h"
prompt_list_response_t* instantiate_prompt_list_response(int include_optional);



prompt_list_response_t* instantiate_prompt_list_response(int include_optional) {
  prompt_list_response_t* prompt_list_response = NULL;
  if (include_optional) {
    prompt_list_response = prompt_list_response_create(
      list_createList()
    );
  } else {
    prompt_list_response = prompt_list_response_create(
      list_createList()
    );
  }

  return prompt_list_response;
}


#ifdef prompt_list_response_MAIN

void test_prompt_list_response(int include_optional) {
    prompt_list_response_t* prompt_list_response_1 = instantiate_prompt_list_response(include_optional);

	cJSON* jsonprompt_list_response_1 = prompt_list_response_convertToJSON(prompt_list_response_1);
	printf("prompt_list_response :\n%s\n", cJSON_Print(jsonprompt_list_response_1));
	prompt_list_response_t* prompt_list_response_2 = prompt_list_response_parseFromJSON(jsonprompt_list_response_1);
	cJSON* jsonprompt_list_response_2 = prompt_list_response_convertToJSON(prompt_list_response_2);
	printf("repeating prompt_list_response:\n%s\n", cJSON_Print(jsonprompt_list_response_2));
}

int main() {
  test_prompt_list_response(1);
  test_prompt_list_response(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_list_response_MAIN
#endif // prompt_list_response_TEST
