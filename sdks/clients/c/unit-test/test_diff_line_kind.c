#ifndef diff_line_kind_TEST
#define diff_line_kind_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define diff_line_kind_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/diff_line_kind.h"
diff_line_kind_t* instantiate_diff_line_kind(int include_optional);



diff_line_kind_t* instantiate_diff_line_kind(int include_optional) {
  diff_line_kind_t* diff_line_kind = NULL;
  if (include_optional) {
    diff_line_kind = diff_line_kind_create(
    );
  } else {
    diff_line_kind = diff_line_kind_create(
    );
  }

  return diff_line_kind;
}


#ifdef diff_line_kind_MAIN

void test_diff_line_kind(int include_optional) {
    diff_line_kind_t* diff_line_kind_1 = instantiate_diff_line_kind(include_optional);

	cJSON* jsondiff_line_kind_1 = diff_line_kind_convertToJSON(diff_line_kind_1);
	printf("diff_line_kind :\n%s\n", cJSON_Print(jsondiff_line_kind_1));
	diff_line_kind_t* diff_line_kind_2 = diff_line_kind_parseFromJSON(jsondiff_line_kind_1);
	cJSON* jsondiff_line_kind_2 = diff_line_kind_convertToJSON(diff_line_kind_2);
	printf("repeating diff_line_kind:\n%s\n", cJSON_Print(jsondiff_line_kind_2));
}

int main() {
  test_diff_line_kind(1);
  test_diff_line_kind(0);

  printf("Hello world \n");
  return 0;
}

#endif // diff_line_kind_MAIN
#endif // diff_line_kind_TEST
