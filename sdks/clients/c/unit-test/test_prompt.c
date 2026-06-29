#ifndef prompt_TEST
#define prompt_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt.h"
prompt_t* instantiate_prompt(int include_optional);



prompt_t* instantiate_prompt(int include_optional) {
  prompt_t* prompt = NULL;
  if (include_optional) {
    prompt = prompt_create(
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      "0",
      "0",
      "0",
      "2013-10-20T19:20:30+01:00"
    );
  } else {
    prompt = prompt_create(
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      "0",
      "0",
      "0",
      "2013-10-20T19:20:30+01:00"
    );
  }

  return prompt;
}


#ifdef prompt_MAIN

void test_prompt(int include_optional) {
    prompt_t* prompt_1 = instantiate_prompt(include_optional);

	cJSON* jsonprompt_1 = prompt_convertToJSON(prompt_1);
	printf("prompt :\n%s\n", cJSON_Print(jsonprompt_1));
	prompt_t* prompt_2 = prompt_parseFromJSON(jsonprompt_1);
	cJSON* jsonprompt_2 = prompt_convertToJSON(prompt_2);
	printf("repeating prompt:\n%s\n", cJSON_Print(jsonprompt_2));
}

int main() {
  test_prompt(1);
  test_prompt(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_MAIN
#endif // prompt_TEST
