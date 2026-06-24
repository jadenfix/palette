#ifndef import_source_http_request_TEST
#define import_source_http_request_TEST

// the following is to include only the main from the first c file
#ifndef TEST_MAIN
#define TEST_MAIN
#define import_source_http_request_MAIN
#endif // TEST_MAIN

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "../external/cJSON.h"

#include "../model/import_source_http_request.h"
import_source_http_request_t* instantiate_import_source_http_request(int include_optional);



import_source_http_request_t* instantiate_import_source_http_request(int include_optional) {
  import_source_http_request_t* import_source_http_request = NULL;
  if (include_optional) {
    import_source_http_request = import_source_http_request_create(
      null,
      "0"
    );
  } else {
    import_source_http_request = import_source_http_request_create(
      null,
      "0"
    );
  }

  return import_source_http_request;
}


#ifdef import_source_http_request_MAIN

void test_import_source_http_request(int include_optional) {
    import_source_http_request_t* import_source_http_request_1 = instantiate_import_source_http_request(include_optional);

	cJSON* jsonimport_source_http_request_1 = import_source_http_request_convertToJSON(import_source_http_request_1);
	printf("import_source_http_request :\n%s\n", cJSON_Print(jsonimport_source_http_request_1));
	import_source_http_request_t* import_source_http_request_2 = import_source_http_request_parseFromJSON(jsonimport_source_http_request_1);
	cJSON* jsonimport_source_http_request_2 = import_source_http_request_convertToJSON(import_source_http_request_2);
	printf("repeating import_source_http_request:\n%s\n", cJSON_Print(jsonimport_source_http_request_2));
}

int main() {
  test_import_source_http_request(1);
  test_import_source_http_request(0);

  printf("Hello world \n");
  return 0;
}

#endif // import_source_http_request_MAIN
#endif // import_source_http_request_TEST
