/*
 * palette_connect_status_response.h
 *
 * 
 */

#ifndef _palette_connect_status_response_H_
#define _palette_connect_status_response_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct palette_connect_status_response_t palette_connect_status_response_t;

#include "palette_connect_status.h"
#include "usage_total.h"



typedef struct palette_connect_status_response_t {
    int first_eval_run; //boolean
    int first_trace_received; //boolean
    int ok; //boolean
    char *project_id; // string
    beater_api_palette_connect_status__e status; //referenced enum
    char *tenant_id; // string
    list_t* totals; //map
    int usage_configured; //boolean

    int _library_owned; // Is the library responsible for freeing this object?
} palette_connect_status_response_t;

__attribute__((deprecated)) palette_connect_status_response_t *palette_connect_status_response_create(
    int first_eval_run,
    int first_trace_received,
    int ok,
    char *project_id,
    beater_api_palette_connect_status__e status,
    char *tenant_id,
    list_t* totals,
    int usage_configured
);

void palette_connect_status_response_free(palette_connect_status_response_t *palette_connect_status_response);

palette_connect_status_response_t *palette_connect_status_response_parseFromJSON(cJSON *palette_connect_status_responseJSON);

cJSON *palette_connect_status_response_convertToJSON(palette_connect_status_response_t *palette_connect_status_response);

#endif /* _palette_connect_status_response_H_ */

