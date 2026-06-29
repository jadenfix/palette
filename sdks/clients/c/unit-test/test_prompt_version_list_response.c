#ifndef prompt_version_list_response_TEST
#define prompt_version_list_response_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_version_list_response_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt_version_list_response.h"
prompt_version_list_response_t* instantiate_prompt_version_list_response(int include_optional);



prompt_version_list_response_t* instantiate_prompt_version_list_response(int include_optional) {
  prompt_version_list_response_t* prompt_version_list_response = NULL;
  if (include_optional) {
    prompt_version_list_response = prompt_version_list_response_create(
      list_createList()
    );
  } else {
    prompt_version_list_response = prompt_version_list_response_create(
      list_createList()
    );
  }

  return prompt_version_list_response;
}


#ifdef prompt_version_list_response_MAIN

void test_prompt_version_list_response(int include_optional) {
    prompt_version_list_response_t* prompt_version_list_response_1 = instantiate_prompt_version_list_response(include_optional);

	cJSON* jsonprompt_version_list_response_1 = prompt_version_list_response_convertToJSON(prompt_version_list_response_1);
	printf("prompt_version_list_response :\n%s\n", cJSON_Print(jsonprompt_version_list_response_1));
	prompt_version_list_response_t* prompt_version_list_response_2 = prompt_version_list_response_parseFromJSON(jsonprompt_version_list_response_1);
	cJSON* jsonprompt_version_list_response_2 = prompt_version_list_response_convertToJSON(prompt_version_list_response_2);
	printf("repeating prompt_version_list_response:\n%s\n", cJSON_Print(jsonprompt_version_list_response_2));
}

int main() {
  test_prompt_version_list_response(1);
  test_prompt_version_list_response(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_version_list_response_MAIN
#endif // prompt_version_list_response_TEST
