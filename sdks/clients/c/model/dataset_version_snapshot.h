/*
 * dataset_version_snapshot.h
 *
 * 
 */

#ifndef _dataset_version_snapshot_H_
#define _dataset_version_snapshot_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct dataset_version_snapshot_t dataset_version_snapshot_t;

#include "dataset_case.h"



typedef struct dataset_version_snapshot_t {
    list_t *cases; //nonprimitive container
    char *corpus_root; // string
    char *created_at; //date time
    char *dataset_id; // string
    char *project_id; // string
    char *tenant_id; // string
    char *version_id; // string

    int _library_owned; // Is the library responsible for freeing this object?
} dataset_version_snapshot_t;

__attribute__((deprecated)) dataset_version_snapshot_t *dataset_version_snapshot_create(
    list_t *cases,
    char *corpus_root,
    char *created_at,
    char *dataset_id,
    char *project_id,
    char *tenant_id,
    char *version_id
);

void dataset_version_snapshot_free(dataset_version_snapshot_t *dataset_version_snapshot);

dataset_version_snapshot_t *dataset_version_snapshot_parseFromJSON(cJSON *dataset_version_snapshotJSON);

cJSON *dataset_version_snapshot_convertToJSON(dataset_version_snapshot_t *dataset_version_snapshot);

#endif /* _dataset_version_snapshot_H_ */

