#ifndef created_prompt_TEST
#define created_prompt_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define created_prompt_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/created_prompt.h"
created_prompt_t* instantiate_created_prompt(int include_optional);

#include "test_prompt.c"
#include "test_prompt_version.c"


created_prompt_t* instantiate_created_prompt(int include_optional) {
  created_prompt_t* created_prompt = NULL;
  if (include_optional) {
    created_prompt = created_prompt_create(
       // false, not to have infinite recursion
      instantiate_prompt(0),
       // false, not to have infinite recursion
      instantiate_prompt_version(0)
    );
  } else {
    created_prompt = created_prompt_create(
      NULL,
      NULL
    );
  }

  return created_prompt;
}


#ifdef created_prompt_MAIN

void test_created_prompt(int include_optional) {
    created_prompt_t* created_prompt_1 = instantiate_created_prompt(include_optional);

	cJSON* jsoncreated_prompt_1 = created_prompt_convertToJSON(created_prompt_1);
	printf("created_prompt :\n%s\n", cJSON_Print(jsoncreated_prompt_1));
	created_prompt_t* created_prompt_2 = created_prompt_parseFromJSON(jsoncreated_prompt_1);
	cJSON* jsoncreated_prompt_2 = created_prompt_convertToJSON(created_prompt_2);
	printf("repeating created_prompt:\n%s\n", cJSON_Print(jsoncreated_prompt_2));
}

int main() {
  test_created_prompt(1);
  test_created_prompt(0);

  printf("Hello world \n");
  return 0;
}

#endif // created_prompt_MAIN
#endif // created_prompt_TEST
