#ifndef signature_TEST
#define signature_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define signature_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/signature.h"
signature_t* instantiate_signature(int include_optional);



signature_t* instantiate_signature(int include_optional) {
  signature_t* signature = NULL;
  if (include_optional) {
    signature = signature_create(
      "0",
      list_createList()
    );
  } else {
    signature = signature_create(
      "0",
      list_createList()
    );
  }

  return signature;
}


#ifdef signature_MAIN

void test_signature(int include_optional) {
    signature_t* signature_1 = instantiate_signature(include_optional);

	cJSON* jsonsignature_1 = signature_convertToJSON(signature_1);
	printf("signature :\n%s\n", cJSON_Print(jsonsignature_1));
	signature_t* signature_2 = signature_parseFromJSON(jsonsignature_1);
	cJSON* jsonsignature_2 = signature_convertToJSON(signature_2);
	printf("repeating signature:\n%s\n", cJSON_Print(jsonsignature_2));
}

int main() {
  test_signature(1);
  test_signature(0);

  printf("Hello world \n");
  return 0;
}

#endif // signature_MAIN
#endif // signature_TEST
