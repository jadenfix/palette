#ifndef connection_link_TEST
#define connection_link_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define connection_link_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/connection_link.h"
connection_link_t* instantiate_connection_link(int include_optional);



connection_link_t* instantiate_connection_link(int include_optional) {
  connection_link_t* connection_link = NULL;
  if (include_optional) {
    connection_link = connection_link_create(
      "0",
      "0",
      "0"
    );
  } else {
    connection_link = connection_link_create(
      "0",
      "0",
      "0"
    );
  }

  return connection_link;
}


#ifdef connection_link_MAIN

void test_connection_link(int include_optional) {
    connection_link_t* connection_link_1 = instantiate_connection_link(include_optional);

	cJSON* jsonconnection_link_1 = connection_link_convertToJSON(connection_link_1);
	printf("connection_link :\n%s\n", cJSON_Print(jsonconnection_link_1));
	connection_link_t* connection_link_2 = connection_link_parseFromJSON(jsonconnection_link_1);
	cJSON* jsonconnection_link_2 = connection_link_convertToJSON(connection_link_2);
	printf("repeating connection_link:\n%s\n", cJSON_Print(jsonconnection_link_2));
}

int main() {
  test_connection_link(1);
  test_connection_link(0);

  printf("Hello world \n");
  return 0;
}

#endif // connection_link_MAIN
#endif // connection_link_TEST
