#ifndef scenario_TEST
#define scenario_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define scenario_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/scenario.h"
scenario_t* instantiate_scenario(int include_optional);

#include "test_perturbation_knobs.c"
#include "test_tenant_scope.c"


scenario_t* instantiate_scenario(int include_optional) {
  scenario_t* scenario = NULL;
  if (include_optional) {
    scenario = scenario_create(
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      beater_api_scenario__tool_error,
       // false, not to have infinite recursion
      instantiate_perturbation_knobs(0),
      0,
      beater_api_scenario__public,
      "0",
       // false, not to have infinite recursion
      instantiate_tenant_scope(0),
      list_createList(),
      "0"
    );
  } else {
    scenario = scenario_create(
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      beater_api_scenario__tool_error,
      NULL,
      0,
      beater_api_scenario__public,
      "0",
      NULL,
      list_createList(),
      "0"
    );
  }

  return scenario;
}


#ifdef scenario_MAIN

void test_scenario(int include_optional) {
    scenario_t* scenario_1 = instantiate_scenario(include_optional);

	cJSON* jsonscenario_1 = scenario_convertToJSON(scenario_1);
	printf("scenario :\n%s\n", cJSON_Print(jsonscenario_1));
	scenario_t* scenario_2 = scenario_parseFromJSON(jsonscenario_1);
	cJSON* jsonscenario_2 = scenario_convertToJSON(scenario_2);
	printf("repeating scenario:\n%s\n", cJSON_Print(jsonscenario_2));
}

int main() {
  test_scenario(1);
  test_scenario(0);

  printf("Hello world \n");
  return 0;
}

#endif // scenario_MAIN
#endif // scenario_TEST
