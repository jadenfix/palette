#ifndef toolkit_TEST
#define toolkit_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define toolkit_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/toolkit.h"
toolkit_t* instantiate_toolkit(int include_optional);



toolkit_t* instantiate_toolkit(int include_optional) {
  toolkit_t* toolkit = NULL;
  if (include_optional) {
    toolkit = toolkit_create(
      list_createList(),
      "0",
      "0",
      1,
      "0",
      0
    );
  } else {
    toolkit = toolkit_create(
      list_createList(),
      "0",
      "0",
      1,
      "0",
      0
    );
  }

  return toolkit;
}


#ifdef toolkit_MAIN

void test_toolkit(int include_optional) {
    toolkit_t* toolkit_1 = instantiate_toolkit(include_optional);

	cJSON* jsontoolkit_1 = toolkit_convertToJSON(toolkit_1);
	printf("toolkit :\n%s\n", cJSON_Print(jsontoolkit_1));
	toolkit_t* toolkit_2 = toolkit_parseFromJSON(jsontoolkit_1);
	cJSON* jsontoolkit_2 = toolkit_convertToJSON(toolkit_2);
	printf("repeating toolkit:\n%s\n", cJSON_Print(jsontoolkit_2));
}

int main() {
  test_toolkit(1);
  test_toolkit(0);

  printf("Hello world \n");
  return 0;
}

#endif // toolkit_MAIN
#endif // toolkit_TEST
