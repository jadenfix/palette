#ifndef palette_connect_status_response_TEST
#define palette_connect_status_response_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define palette_connect_status_response_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/palette_connect_status_response.h"
palette_connect_status_response_t* instantiate_palette_connect_status_response(int include_optional);



palette_connect_status_response_t* instantiate_palette_connect_status_response(int include_optional) {
  palette_connect_status_response_t* palette_connect_status_response = NULL;
  if (include_optional) {
    palette_connect_status_response = palette_connect_status_response_create(
      1,
      1,
      1,
      "0",
      beater_api_palette_connect_status_response__connected,
      "0",
      list_createList(),
      1
    );
  } else {
    palette_connect_status_response = palette_connect_status_response_create(
      1,
      1,
      1,
      "0",
      beater_api_palette_connect_status_response__connected,
      "0",
      list_createList(),
      1
    );
  }

  return palette_connect_status_response;
}


#ifdef palette_connect_status_response_MAIN

void test_palette_connect_status_response(int include_optional) {
    palette_connect_status_response_t* palette_connect_status_response_1 = instantiate_palette_connect_status_response(include_optional);

	cJSON* jsonpalette_connect_status_response_1 = palette_connect_status_response_convertToJSON(palette_connect_status_response_1);
	printf("palette_connect_status_response :\n%s\n", cJSON_Print(jsonpalette_connect_status_response_1));
	palette_connect_status_response_t* palette_connect_status_response_2 = palette_connect_status_response_parseFromJSON(jsonpalette_connect_status_response_1);
	cJSON* jsonpalette_connect_status_response_2 = palette_connect_status_response_convertToJSON(palette_connect_status_response_2);
	printf("repeating palette_connect_status_response:\n%s\n", cJSON_Print(jsonpalette_connect_status_response_2));
}

int main() {
  test_palette_connect_status_response(1);
  test_palette_connect_status_response(0);

  printf("Hello world \n");
  return 0;
}

#endif // palette_connect_status_response_MAIN
#endif // palette_connect_status_response_TEST
