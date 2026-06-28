#ifndef experiment_comparison_TEST
#define experiment_comparison_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define experiment_comparison_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/experiment_comparison.h"
experiment_comparison_t* instantiate_experiment_comparison(int include_optional);



experiment_comparison_t* instantiate_experiment_comparison(int include_optional) {
  experiment_comparison_t* experiment_comparison = NULL;
  if (include_optional) {
    experiment_comparison = experiment_comparison_create(
      1.337,
      1.337,
      1.337,
      1.337,
      1.337,
      beater_api_experiment_comparison__pass,
      1.337,
      1.337,
      0,
      beater_api_experiment_comparison__paired_t
    );
  } else {
    experiment_comparison = experiment_comparison_create(
      1.337,
      1.337,
      1.337,
      1.337,
      1.337,
      beater_api_experiment_comparison__pass,
      1.337,
      1.337,
      0,
      beater_api_experiment_comparison__paired_t
    );
  }

  return experiment_comparison;
}


#ifdef experiment_comparison_MAIN

void test_experiment_comparison(int include_optional) {
    experiment_comparison_t* experiment_comparison_1 = instantiate_experiment_comparison(include_optional);

	cJSON* jsonexperiment_comparison_1 = experiment_comparison_convertToJSON(experiment_comparison_1);
	printf("experiment_comparison :\n%s\n", cJSON_Print(jsonexperiment_comparison_1));
	experiment_comparison_t* experiment_comparison_2 = experiment_comparison_parseFromJSON(jsonexperiment_comparison_1);
	cJSON* jsonexperiment_comparison_2 = experiment_comparison_convertToJSON(experiment_comparison_2);
	printf("repeating experiment_comparison:\n%s\n", cJSON_Print(jsonexperiment_comparison_2));
}

int main() {
  test_experiment_comparison(1);
  test_experiment_comparison(0);

  printf("Hello world \n");
  return 0;
}

#endif // experiment_comparison_MAIN
#endif // experiment_comparison_TEST
