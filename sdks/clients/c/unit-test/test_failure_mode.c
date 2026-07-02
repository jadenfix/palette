#ifndef failure_mode_TEST
#define failure_mode_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define failure_mode_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/failure_mode.h"
failure_mode_t* instantiate_failure_mode(int include_optional);



failure_mode_t* instantiate_failure_mode(int include_optional) {
  failure_mode_t* failure_mode = NULL;
  if (include_optional) {
    failure_mode = failure_mode_create(
    );
  } else {
    failure_mode = failure_mode_create(
    );
  }

  return failure_mode;
}


#ifdef failure_mode_MAIN

void test_failure_mode(int include_optional) {
    failure_mode_t* failure_mode_1 = instantiate_failure_mode(include_optional);

	cJSON* jsonfailure_mode_1 = failure_mode_convertToJSON(failure_mode_1);
	printf("failure_mode :\n%s\n", cJSON_Print(jsonfailure_mode_1));
	failure_mode_t* failure_mode_2 = failure_mode_parseFromJSON(jsonfailure_mode_1);
	cJSON* jsonfailure_mode_2 = failure_mode_convertToJSON(failure_mode_2);
	printf("repeating failure_mode:\n%s\n", cJSON_Print(jsonfailure_mode_2));
}

int main() {
  test_failure_mode(1);
  test_failure_mode(0);

  printf("Hello world \n");
  return 0;
}

#endif // failure_mode_MAIN
#endif // failure_mode_TEST
