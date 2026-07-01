#ifndef connector_tool_TEST
#define connector_tool_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define connector_tool_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/connector_tool.h"
connector_tool_t* instantiate_connector_tool(int include_optional);



connector_tool_t* instantiate_connector_tool(int include_optional) {
  connector_tool_t* connector_tool = NULL;
  if (include_optional) {
    connector_tool = connector_tool_create(
      "0",
      0,
      "0",
      1,
      "0",
      list_createList(),
      "0"
    );
  } else {
    connector_tool = connector_tool_create(
      "0",
      0,
      "0",
      1,
      "0",
      list_createList(),
      "0"
    );
  }

  return connector_tool;
}


#ifdef connector_tool_MAIN

void test_connector_tool(int include_optional) {
    connector_tool_t* connector_tool_1 = instantiate_connector_tool(include_optional);

	cJSON* jsonconnector_tool_1 = connector_tool_convertToJSON(connector_tool_1);
	printf("connector_tool :\n%s\n", cJSON_Print(jsonconnector_tool_1));
	connector_tool_t* connector_tool_2 = connector_tool_parseFromJSON(jsonconnector_tool_1);
	cJSON* jsonconnector_tool_2 = connector_tool_convertToJSON(connector_tool_2);
	printf("repeating connector_tool:\n%s\n", cJSON_Print(jsonconnector_tool_2));
}

int main() {
  test_connector_tool(1);
  test_connector_tool(0);

  printf("Hello world \n");
  return 0;
}

#endif // connector_tool_MAIN
#endif // connector_tool_TEST
