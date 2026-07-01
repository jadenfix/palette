#ifndef create_prompt_request_TEST
#define create_prompt_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define create_prompt_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/create_prompt_request.h"
create_prompt_request_t* instantiate_create_prompt_request(int include_optional);

#include "test_prompt_template.c"


create_prompt_request_t* instantiate_create_prompt_request(int include_optional) {
  create_prompt_request_t* create_prompt_request = NULL;
  if (include_optional) {
    create_prompt_request = create_prompt_request_create(
      "0",
      "0",
      "0",
      "0",
       // false, not to have infinite recursion
      instantiate_prompt_template(0)
    );
  } else {
    create_prompt_request = create_prompt_request_create(
      "0",
      "0",
      "0",
      "0",
      NULL
    );
  }

  return create_prompt_request;
}


#ifdef create_prompt_request_MAIN

void test_create_prompt_request(int include_optional) {
    create_prompt_request_t* create_prompt_request_1 = instantiate_create_prompt_request(include_optional);

	cJSON* jsoncreate_prompt_request_1 = create_prompt_request_convertToJSON(create_prompt_request_1);
	printf("create_prompt_request :\n%s\n", cJSON_Print(jsoncreate_prompt_request_1));
	create_prompt_request_t* create_prompt_request_2 = create_prompt_request_parseFromJSON(jsoncreate_prompt_request_1);
	cJSON* jsoncreate_prompt_request_2 = create_prompt_request_convertToJSON(create_prompt_request_2);
	printf("repeating create_prompt_request:\n%s\n", cJSON_Print(jsoncreate_prompt_request_2));
}

int main() {
  test_create_prompt_request(1);
  test_create_prompt_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // create_prompt_request_MAIN
#endif // create_prompt_request_TEST
