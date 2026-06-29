#ifndef prompt_version_TEST
#define prompt_version_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_version_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt_version.h"
prompt_version_t* instantiate_prompt_version(int include_optional);

#include "test_prompt_version_metadata.c"
#include "test_prompt_template.c"


prompt_version_t* instantiate_prompt_version(int include_optional) {
  prompt_version_t* prompt_version = NULL;
  if (include_optional) {
    prompt_version = prompt_version_create(
       // false, not to have infinite recursion
      instantiate_prompt_version_metadata(0),
      "0",
      "0",
       // false, not to have infinite recursion
      instantiate_prompt_template(0),
      "0",
      "0",
      0
    );
  } else {
    prompt_version = prompt_version_create(
      NULL,
      "0",
      "0",
      NULL,
      "0",
      "0",
      0
    );
  }

  return prompt_version;
}


#ifdef prompt_version_MAIN

void test_prompt_version(int include_optional) {
    prompt_version_t* prompt_version_1 = instantiate_prompt_version(include_optional);

	cJSON* jsonprompt_version_1 = prompt_version_convertToJSON(prompt_version_1);
	printf("prompt_version :\n%s\n", cJSON_Print(jsonprompt_version_1));
	prompt_version_t* prompt_version_2 = prompt_version_parseFromJSON(jsonprompt_version_1);
	cJSON* jsonprompt_version_2 = prompt_version_convertToJSON(prompt_version_2);
	printf("repeating prompt_version:\n%s\n", cJSON_Print(jsonprompt_version_2));
}

int main() {
  test_prompt_version(1);
  test_prompt_version(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_version_MAIN
#endif // prompt_version_TEST
