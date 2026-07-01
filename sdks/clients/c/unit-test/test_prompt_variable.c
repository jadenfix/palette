#ifndef prompt_variable_TEST
#define prompt_variable_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_variable_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt_variable.h"
prompt_variable_t* instantiate_prompt_variable(int include_optional);



prompt_variable_t* instantiate_prompt_variable(int include_optional) {
  prompt_variable_t* prompt_variable = NULL;
  if (include_optional) {
    prompt_variable = prompt_variable_create(
      "0",
      "0",
      "0",
      1
    );
  } else {
    prompt_variable = prompt_variable_create(
      "0",
      "0",
      "0",
      1
    );
  }

  return prompt_variable;
}


#ifdef prompt_variable_MAIN

void test_prompt_variable(int include_optional) {
    prompt_variable_t* prompt_variable_1 = instantiate_prompt_variable(include_optional);

	cJSON* jsonprompt_variable_1 = prompt_variable_convertToJSON(prompt_variable_1);
	printf("prompt_variable :\n%s\n", cJSON_Print(jsonprompt_variable_1));
	prompt_variable_t* prompt_variable_2 = prompt_variable_parseFromJSON(jsonprompt_variable_1);
	cJSON* jsonprompt_variable_2 = prompt_variable_convertToJSON(prompt_variable_2);
	printf("repeating prompt_variable:\n%s\n", cJSON_Print(jsonprompt_variable_2));
}

int main() {
  test_prompt_variable(1);
  test_prompt_variable(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_variable_MAIN
#endif // prompt_variable_TEST
