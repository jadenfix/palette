/*
 * prompt_version_metadata.h
 *
 * 
 */

#ifndef _prompt_version_metadata_H_
#define _prompt_version_metadata_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct prompt_version_metadata_t prompt_version_metadata_t;




typedef struct prompt_version_metadata_t {
    char *created_at; //date time
    char *created_by; // string
    char *message; // string

    int _library_owned; // Is the library responsible for freeing this object?
} prompt_version_metadata_t;

__attribute__((deprecated)) prompt_version_metadata_t *prompt_version_metadata_create(
    char *created_at,
    char *created_by,
    char *message
);

void prompt_version_metadata_free(prompt_version_metadata_t *prompt_version_metadata);

prompt_version_metadata_t *prompt_version_metadata_parseFromJSON(cJSON *prompt_version_metadataJSON);

cJSON *prompt_version_metadata_convertToJSON(prompt_version_metadata_t *prompt_version_metadata);

#endif /* _prompt_version_metadata_H_ */

