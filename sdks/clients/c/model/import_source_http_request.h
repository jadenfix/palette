/*
 * import_source_http_request.h
 *
 * Request body for the unified import endpoint. The &#x60;source&#x60; field selects a registered [&#x60;beater_ingest::SourceImporter&#x60;] (e.g. &#x60;temporal_history&#x60;, &#x60;native&#x60;); &#x60;payload&#x60; is that source&#39;s document (Temporal &#x60;History&#x60; JSON, a native span list, …). Everything flows through the same downstream ingest pipeline as OTLP — there are no source-specific routes.
 */

#ifndef _import_source_http_request_H_
#define _import_source_http_request_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct import_source_http_request_t import_source_http_request_t;

#include "any_type.h"



typedef struct import_source_http_request_t {
    any_type_t *payload; // custom
    char *source; // string

    int _library_owned; // Is the library responsible for freeing this object?
} import_source_http_request_t;

__attribute__((deprecated)) import_source_http_request_t *import_source_http_request_create(
    any_type_t *payload,
    char *source
);

void import_source_http_request_free(import_source_http_request_t *import_source_http_request);

import_source_http_request_t *import_source_http_request_parseFromJSON(cJSON *import_source_http_requestJSON);

cJSON *import_source_http_request_convertToJSON(import_source_http_request_t *import_source_http_request);

#endif /* _import_source_http_request_H_ */

