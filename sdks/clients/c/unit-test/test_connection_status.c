#ifndef connection_status_TEST
#define connection_status_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define connection_status_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/connection_status.h"
connection_status_t* instantiate_connection_status(int include_optional);



connection_status_t* instantiate_connection_status(int include_optional) {
  connection_status_t* connection_status = NULL;
  if (include_optional) {
    connection_status = connection_status_create(
      1,
      "0",
      "0",
      "0"
    );
  } else {
    connection_status = connection_status_create(
      1,
      "0",
      "0",
      "0"
    );
  }

  return connection_status;
}


#ifdef connection_status_MAIN

void test_connection_status(int include_optional) {
    connection_status_t* connection_status_1 = instantiate_connection_status(include_optional);

	cJSON* jsonconnection_status_1 = connection_status_convertToJSON(connection_status_1);
	printf("connection_status :\n%s\n", cJSON_Print(jsonconnection_status_1));
	connection_status_t* connection_status_2 = connection_status_parseFromJSON(jsonconnection_status_1);
	cJSON* jsonconnection_status_2 = connection_status_convertToJSON(connection_status_2);
	printf("repeating connection_status:\n%s\n", cJSON_Print(jsonconnection_status_2));
}

int main() {
  test_connection_status(1);
  test_connection_status(0);

  printf("Hello world \n");
  return 0;
}

#endif // connection_status_MAIN
#endif // connection_status_TEST
