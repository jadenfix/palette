#ifndef tool_execution_TEST
#define tool_execution_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define tool_execution_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/tool_execution.h"
tool_execution_t* instantiate_tool_execution(int include_optional);



tool_execution_t* instantiate_tool_execution(int include_optional) {
  tool_execution_t* tool_execution = NULL;
  if (include_optional) {
    tool_execution = tool_execution_create(
      0,
      "0",
      "0",
      1
    );
  } else {
    tool_execution = tool_execution_create(
      0,
      "0",
      "0",
      1
    );
  }

  return tool_execution;
}


#ifdef tool_execution_MAIN

void test_tool_execution(int include_optional) {
    tool_execution_t* tool_execution_1 = instantiate_tool_execution(include_optional);

	cJSON* jsontool_execution_1 = tool_execution_convertToJSON(tool_execution_1);
	printf("tool_execution :\n%s\n", cJSON_Print(jsontool_execution_1));
	tool_execution_t* tool_execution_2 = tool_execution_parseFromJSON(jsontool_execution_1);
	cJSON* jsontool_execution_2 = tool_execution_convertToJSON(tool_execution_2);
	printf("repeating tool_execution:\n%s\n", cJSON_Print(jsontool_execution_2));
}

int main() {
  test_tool_execution(1);
  test_tool_execution(0);

  printf("Hello world \n");
  return 0;
}

#endif // tool_execution_MAIN
#endif // tool_execution_TEST
