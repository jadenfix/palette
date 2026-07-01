#ifndef add_prompt_version_request_TEST
#define add_prompt_version_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define add_prompt_version_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/add_prompt_version_request.h"
add_prompt_version_request_t* instantiate_add_prompt_version_request(int include_optional);

#include "test_prompt_template.c"


add_prompt_version_request_t* instantiate_add_prompt_version_request(int include_optional) {
  add_prompt_version_request_t* add_prompt_version_request = NULL;
  if (include_optional) {
    add_prompt_version_request = add_prompt_version_request_create(
      "0",
      "0",
       // false, not to have infinite recursion
      instantiate_prompt_template(0)
    );
  } else {
    add_prompt_version_request = add_prompt_version_request_create(
      "0",
      "0",
      NULL
    );
  }

  return add_prompt_version_request;
}


#ifdef add_prompt_version_request_MAIN

void test_add_prompt_version_request(int include_optional) {
    add_prompt_version_request_t* add_prompt_version_request_1 = instantiate_add_prompt_version_request(include_optional);

	cJSON* jsonadd_prompt_version_request_1 = add_prompt_version_request_convertToJSON(add_prompt_version_request_1);
	printf("add_prompt_version_request :\n%s\n", cJSON_Print(jsonadd_prompt_version_request_1));
	add_prompt_version_request_t* add_prompt_version_request_2 = add_prompt_version_request_parseFromJSON(jsonadd_prompt_version_request_1);
	cJSON* jsonadd_prompt_version_request_2 = add_prompt_version_request_convertToJSON(add_prompt_version_request_2);
	printf("repeating add_prompt_version_request:\n%s\n", cJSON_Print(jsonadd_prompt_version_request_2));
}

int main() {
  test_add_prompt_version_request(1);
  test_add_prompt_version_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // add_prompt_version_request_MAIN
#endif // add_prompt_version_request_TEST
