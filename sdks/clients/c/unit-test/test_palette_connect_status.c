#ifndef palette_connect_status_TEST
#define palette_connect_status_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define palette_connect_status_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/palette_connect_status.h"
palette_connect_status_t* instantiate_palette_connect_status(int include_optional);



palette_connect_status_t* instantiate_palette_connect_status(int include_optional) {
  palette_connect_status_t* palette_connect_status = NULL;
  if (include_optional) {
    palette_connect_status = palette_connect_status_create(
    );
  } else {
    palette_connect_status = palette_connect_status_create(
    );
  }

  return palette_connect_status;
}


#ifdef palette_connect_status_MAIN

void test_palette_connect_status(int include_optional) {
    palette_connect_status_t* palette_connect_status_1 = instantiate_palette_connect_status(include_optional);

	cJSON* jsonpalette_connect_status_1 = palette_connect_status_convertToJSON(palette_connect_status_1);
	printf("palette_connect_status :\n%s\n", cJSON_Print(jsonpalette_connect_status_1));
	palette_connect_status_t* palette_connect_status_2 = palette_connect_status_parseFromJSON(jsonpalette_connect_status_1);
	cJSON* jsonpalette_connect_status_2 = palette_connect_status_convertToJSON(palette_connect_status_2);
	printf("repeating palette_connect_status:\n%s\n", cJSON_Print(jsonpalette_connect_status_2));
}

int main() {
  test_palette_connect_status(1);
  test_palette_connect_status(0);

  printf("Hello world \n");
  return 0;
}

#endif // palette_connect_status_MAIN
#endif // palette_connect_status_TEST
