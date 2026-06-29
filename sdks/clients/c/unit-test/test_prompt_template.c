#ifndef prompt_template_TEST
#define prompt_template_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_template_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt_template.h"
prompt_template_t* instantiate_prompt_template(int include_optional);



prompt_template_t* instantiate_prompt_template(int include_optional) {
  prompt_template_t* prompt_template = NULL;
  if (include_optional) {
    prompt_template = prompt_template_create(
      "0",
      list_createList(),
      list_createList()
    );
  } else {
    prompt_template = prompt_template_create(
      "0",
      list_createList(),
      list_createList()
    );
  }

  return prompt_template;
}


#ifdef prompt_template_MAIN

void test_prompt_template(int include_optional) {
    prompt_template_t* prompt_template_1 = instantiate_prompt_template(include_optional);

	cJSON* jsonprompt_template_1 = prompt_template_convertToJSON(prompt_template_1);
	printf("prompt_template :\n%s\n", cJSON_Print(jsonprompt_template_1));
	prompt_template_t* prompt_template_2 = prompt_template_parseFromJSON(jsonprompt_template_1);
	cJSON* jsonprompt_template_2 = prompt_template_convertToJSON(prompt_template_2);
	printf("repeating prompt_template:\n%s\n", cJSON_Print(jsonprompt_template_2));
}

int main() {
  test_prompt_template(1);
  test_prompt_template(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_template_MAIN
#endif // prompt_template_TEST
