/*
 * diff_line.h
 *
 * 
 */

#ifndef _diff_line_H_
#define _diff_line_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct diff_line_t diff_line_t;

#include "diff_line_kind.h"



typedef struct diff_line_t {
    beater_api_diff_line_kind__e kind; //referenced enum
    int new_line; //numeric
    int old_line; //numeric
    char *text; // string

    int _library_owned; // Is the library responsible for freeing this object?
} diff_line_t;

__attribute__((deprecated)) diff_line_t *diff_line_create(
    beater_api_diff_line_kind__e kind,
    int new_line,
    int old_line,
    char *text
);

void diff_line_free(diff_line_t *diff_line);

diff_line_t *diff_line_parseFromJSON(cJSON *diff_lineJSON);

cJSON *diff_line_convertToJSON(diff_line_t *diff_line);

#endif /* _diff_line_H_ */

