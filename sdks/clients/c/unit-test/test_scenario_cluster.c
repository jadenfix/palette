#ifndef scenario_cluster_TEST
#define scenario_cluster_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define scenario_cluster_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/scenario_cluster.h"
scenario_cluster_t* instantiate_scenario_cluster(int include_optional);

#include "test_signature.c"


scenario_cluster_t* instantiate_scenario_cluster(int include_optional) {
  scenario_cluster_t* scenario_cluster = NULL;
  if (include_optional) {
    scenario_cluster = scenario_cluster_create(
      beater_api_scenario_cluster__tool_error,
      "0",
      list_createList(),
       // false, not to have infinite recursion
      instantiate_signature(0),
      0
    );
  } else {
    scenario_cluster = scenario_cluster_create(
      beater_api_scenario_cluster__tool_error,
      "0",
      list_createList(),
      NULL,
      0
    );
  }

  return scenario_cluster;
}


#ifdef scenario_cluster_MAIN

void test_scenario_cluster(int include_optional) {
    scenario_cluster_t* scenario_cluster_1 = instantiate_scenario_cluster(include_optional);

	cJSON* jsonscenario_cluster_1 = scenario_cluster_convertToJSON(scenario_cluster_1);
	printf("scenario_cluster :\n%s\n", cJSON_Print(jsonscenario_cluster_1));
	scenario_cluster_t* scenario_cluster_2 = scenario_cluster_parseFromJSON(jsonscenario_cluster_1);
	cJSON* jsonscenario_cluster_2 = scenario_cluster_convertToJSON(scenario_cluster_2);
	printf("repeating scenario_cluster:\n%s\n", cJSON_Print(jsonscenario_cluster_2));
}

int main() {
  test_scenario_cluster(1);
  test_scenario_cluster(0);

  printf("Hello world \n");
  return 0;
}

#endif // scenario_cluster_MAIN
#endif // scenario_cluster_TEST
