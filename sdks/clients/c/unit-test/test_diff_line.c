#ifndef diff_line_TEST
#define diff_line_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define diff_line_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/diff_line.h"
diff_line_t* instantiate_diff_line(int include_optional);



diff_line_t* instantiate_diff_line(int include_optional) {
  diff_line_t* diff_line = NULL;
  if (include_optional) {
    diff_line = diff_line_create(
      beater_api_diff_line__unchanged,
      0,
      0,
      "0"
    );
  } else {
    diff_line = diff_line_create(
      beater_api_diff_line__unchanged,
      0,
      0,
      "0"
    );
  }

  return diff_line;
}


#ifdef diff_line_MAIN

void test_diff_line(int include_optional) {
    diff_line_t* diff_line_1 = instantiate_diff_line(include_optional);

	cJSON* jsondiff_line_1 = diff_line_convertToJSON(diff_line_1);
	printf("diff_line :\n%s\n", cJSON_Print(jsondiff_line_1));
	diff_line_t* diff_line_2 = diff_line_parseFromJSON(jsondiff_line_1);
	cJSON* jsondiff_line_2 = diff_line_convertToJSON(diff_line_2);
	printf("repeating diff_line:\n%s\n", cJSON_Print(jsondiff_line_2));
}

int main() {
  test_diff_line(1);
  test_diff_line(0);

  printf("Hello world \n");
  return 0;
}

#endif // diff_line_MAIN
#endif // diff_line_TEST
