#ifndef prompt_version_diff_TEST
#define prompt_version_diff_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_version_diff_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt_version_diff.h"
prompt_version_diff_t* instantiate_prompt_version_diff(int include_optional);



prompt_version_diff_t* instantiate_prompt_version_diff(int include_optional) {
  prompt_version_diff_t* prompt_version_diff = NULL;
  if (include_optional) {
    prompt_version_diff = prompt_version_diff_create(
      "0",
      list_createList(),
      "0"
    );
  } else {
    prompt_version_diff = prompt_version_diff_create(
      "0",
      list_createList(),
      "0"
    );
  }

  return prompt_version_diff;
}


#ifdef prompt_version_diff_MAIN

void test_prompt_version_diff(int include_optional) {
    prompt_version_diff_t* prompt_version_diff_1 = instantiate_prompt_version_diff(include_optional);

	cJSON* jsonprompt_version_diff_1 = prompt_version_diff_convertToJSON(prompt_version_diff_1);
	printf("prompt_version_diff :\n%s\n", cJSON_Print(jsonprompt_version_diff_1));
	prompt_version_diff_t* prompt_version_diff_2 = prompt_version_diff_parseFromJSON(jsonprompt_version_diff_1);
	cJSON* jsonprompt_version_diff_2 = prompt_version_diff_convertToJSON(prompt_version_diff_2);
	printf("repeating prompt_version_diff:\n%s\n", cJSON_Print(jsonprompt_version_diff_2));
}

int main() {
  test_prompt_version_diff(1);
  test_prompt_version_diff(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_version_diff_MAIN
#endif // prompt_version_diff_TEST
