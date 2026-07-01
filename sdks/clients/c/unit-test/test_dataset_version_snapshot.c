#ifndef dataset_version_snapshot_TEST
#define dataset_version_snapshot_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define dataset_version_snapshot_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/dataset_version_snapshot.h"
dataset_version_snapshot_t* instantiate_dataset_version_snapshot(int include_optional);



dataset_version_snapshot_t* instantiate_dataset_version_snapshot(int include_optional) {
  dataset_version_snapshot_t* dataset_version_snapshot = NULL;
  if (include_optional) {
    dataset_version_snapshot = dataset_version_snapshot_create(
      list_createList(),
      "0",
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      "0",
      "0"
    );
  } else {
    dataset_version_snapshot = dataset_version_snapshot_create(
      list_createList(),
      "0",
      "2013-10-20T19:20:30+01:00",
      "0",
      "0",
      "0",
      "0"
    );
  }

  return dataset_version_snapshot;
}


#ifdef dataset_version_snapshot_MAIN

void test_dataset_version_snapshot(int include_optional) {
    dataset_version_snapshot_t* dataset_version_snapshot_1 = instantiate_dataset_version_snapshot(include_optional);

	cJSON* jsondataset_version_snapshot_1 = dataset_version_snapshot_convertToJSON(dataset_version_snapshot_1);
	printf("dataset_version_snapshot :\n%s\n", cJSON_Print(jsondataset_version_snapshot_1));
	dataset_version_snapshot_t* dataset_version_snapshot_2 = dataset_version_snapshot_parseFromJSON(jsondataset_version_snapshot_1);
	cJSON* jsondataset_version_snapshot_2 = dataset_version_snapshot_convertToJSON(dataset_version_snapshot_2);
	printf("repeating dataset_version_snapshot:\n%s\n", cJSON_Print(jsondataset_version_snapshot_2));
}

int main() {
  test_dataset_version_snapshot(1);
  test_dataset_version_snapshot(0);

  printf("Hello world \n");
  return 0;
}

#endif // dataset_version_snapshot_MAIN
#endif // dataset_version_snapshot_TEST
