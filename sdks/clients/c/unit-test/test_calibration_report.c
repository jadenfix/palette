#ifndef calibration_report_TEST
#define calibration_report_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define calibration_report_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/calibration_report.h"
calibration_report_t* instantiate_calibration_report(int include_optional);

#include "test_calibration_confusion.c"
#include "test_calibration_policy.c"


calibration_report_t* instantiate_calibration_report(int include_optional) {
  calibration_report_t* calibration_report = NULL;
  if (include_optional) {
    calibration_report = calibration_report_create(
      1.337,
      "0",
      1.337,
      1.337,
      1.337,
       // false, not to have infinite recursion
      instantiate_calibration_confusion(0),
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      "0",
      "0",
      1.337,
      1.337,
      list_createList(),
      1.337,
      1.337,
      1.337,
       // false, not to have infinite recursion
      instantiate_calibration_policy(0),
      "0",
      list_createList(),
      0,
      "0"
    );
  } else {
    calibration_report = calibration_report_create(
      1.337,
      "0",
      1.337,
      1.337,
      1.337,
      NULL,
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      "0",
      "0",
      1.337,
      1.337,
      list_createList(),
      1.337,
      1.337,
      1.337,
      NULL,
      "0",
      list_createList(),
      0,
      "0"
    );
  }

  return calibration_report;
}


#ifdef calibration_report_MAIN

void test_calibration_report(int include_optional) {
    calibration_report_t* calibration_report_1 = instantiate_calibration_report(include_optional);

	cJSON* jsoncalibration_report_1 = calibration_report_convertToJSON(calibration_report_1);
	printf("calibration_report :\n%s\n", cJSON_Print(jsoncalibration_report_1));
	calibration_report_t* calibration_report_2 = calibration_report_parseFromJSON(jsoncalibration_report_1);
	cJSON* jsoncalibration_report_2 = calibration_report_convertToJSON(calibration_report_2);
	printf("repeating calibration_report:\n%s\n", cJSON_Print(jsoncalibration_report_2));
}

int main() {
  test_calibration_report(1);
  test_calibration_report(0);

  printf("Hello world \n");
  return 0;
}

#endif // calibration_report_MAIN
#endif // calibration_report_TEST
