#ifndef prompt_version_metadata_TEST
#define prompt_version_metadata_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define prompt_version_metadata_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/prompt_version_metadata.h"
prompt_version_metadata_t* instantiate_prompt_version_metadata(int include_optional);



prompt_version_metadata_t* instantiate_prompt_version_metadata(int include_optional) {
  prompt_version_metadata_t* prompt_version_metadata = NULL;
  if (include_optional) {
    prompt_version_metadata = prompt_version_metadata_create(
      "2013-10-20T19:20:30+01:00",
      "0",
      "0"
    );
  } else {
    prompt_version_metadata = prompt_version_metadata_create(
      "2013-10-20T19:20:30+01:00",
      "0",
      "0"
    );
  }

  return prompt_version_metadata;
}


#ifdef prompt_version_metadata_MAIN

void test_prompt_version_metadata(int include_optional) {
    prompt_version_metadata_t* prompt_version_metadata_1 = instantiate_prompt_version_metadata(include_optional);

	cJSON* jsonprompt_version_metadata_1 = prompt_version_metadata_convertToJSON(prompt_version_metadata_1);
	printf("prompt_version_metadata :\n%s\n", cJSON_Print(jsonprompt_version_metadata_1));
	prompt_version_metadata_t* prompt_version_metadata_2 = prompt_version_metadata_parseFromJSON(jsonprompt_version_metadata_1);
	cJSON* jsonprompt_version_metadata_2 = prompt_version_metadata_convertToJSON(prompt_version_metadata_2);
	printf("repeating prompt_version_metadata:\n%s\n", cJSON_Print(jsonprompt_version_metadata_2));
}

int main() {
  test_prompt_version_metadata(1);
  test_prompt_version_metadata(0);

  printf("Hello world \n");
  return 0;
}

#endif // prompt_version_metadata_MAIN
#endif // prompt_version_metadata_TEST
