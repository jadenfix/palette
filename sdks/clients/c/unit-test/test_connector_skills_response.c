#ifndef connector_skills_response_TEST
#define connector_skills_response_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define connector_skills_response_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/connector_skills_response.h"
connector_skills_response_t* instantiate_connector_skills_response(int include_optional);



connector_skills_response_t* instantiate_connector_skills_response(int include_optional) {
  connector_skills_response_t* connector_skills_response = NULL;
  if (include_optional) {
    connector_skills_response = connector_skills_response_create(
      "0",
      "0"
    );
  } else {
    connector_skills_response = connector_skills_response_create(
      "0",
      "0"
    );
  }

  return connector_skills_response;
}


#ifdef connector_skills_response_MAIN

void test_connector_skills_response(int include_optional) {
    connector_skills_response_t* connector_skills_response_1 = instantiate_connector_skills_response(include_optional);

	cJSON* jsonconnector_skills_response_1 = connector_skills_response_convertToJSON(connector_skills_response_1);
	printf("connector_skills_response :\n%s\n", cJSON_Print(jsonconnector_skills_response_1));
	connector_skills_response_t* connector_skills_response_2 = connector_skills_response_parseFromJSON(jsonconnector_skills_response_1);
	cJSON* jsonconnector_skills_response_2 = connector_skills_response_convertToJSON(connector_skills_response_2);
	printf("repeating connector_skills_response:\n%s\n", cJSON_Print(jsonconnector_skills_response_2));
}

int main() {
  test_connector_skills_response(1);
  test_connector_skills_response(0);

  printf("Hello world \n");
  return 0;
}

#endif // connector_skills_response_MAIN
#endif // connector_skills_response_TEST
