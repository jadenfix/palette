#ifndef perturbation_knobs_TEST
#define perturbation_knobs_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define perturbation_knobs_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/perturbation_knobs.h"
perturbation_knobs_t* instantiate_perturbation_knobs(int include_optional);



perturbation_knobs_t* instantiate_perturbation_knobs(int include_optional) {
  perturbation_knobs_t* perturbation_knobs = NULL;
  if (include_optional) {
    perturbation_knobs = perturbation_knobs_create(
      1,
      1,
      1,
      1,
      1,
      1
    );
  } else {
    perturbation_knobs = perturbation_knobs_create(
      1,
      1,
      1,
      1,
      1,
      1
    );
  }

  return perturbation_knobs;
}


#ifdef perturbation_knobs_MAIN

void test_perturbation_knobs(int include_optional) {
    perturbation_knobs_t* perturbation_knobs_1 = instantiate_perturbation_knobs(include_optional);

	cJSON* jsonperturbation_knobs_1 = perturbation_knobs_convertToJSON(perturbation_knobs_1);
	printf("perturbation_knobs :\n%s\n", cJSON_Print(jsonperturbation_knobs_1));
	perturbation_knobs_t* perturbation_knobs_2 = perturbation_knobs_parseFromJSON(jsonperturbation_knobs_1);
	cJSON* jsonperturbation_knobs_2 = perturbation_knobs_convertToJSON(perturbation_knobs_2);
	printf("repeating perturbation_knobs:\n%s\n", cJSON_Print(jsonperturbation_knobs_2));
}

int main() {
  test_perturbation_knobs(1);
  test_perturbation_knobs(0);

  printf("Hello world \n");
  return 0;
}

#endif // perturbation_knobs_MAIN
#endif // perturbation_knobs_TEST
