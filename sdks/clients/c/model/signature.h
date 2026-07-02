/*
 * signature.h
 *
 * A structural fingerprint of a trace&#39;s failure shape.  Two traces with the same ordered failing-span shingles share a [&#x60;Signature&#x60;] (and therefore the same [&#x60;Signature::hash&#x60;]). The &#x60;shingles&#x60; set is also used for Jaccard similarity during clustering.
 */

#ifndef _signature_H_
#define _signature_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct signature_t signature_t;




typedef struct signature_t {
    char *hash; // string
    list_t *shingles; //primitive container

    int _library_owned; // Is the library responsible for freeing this object?
} signature_t;

__attribute__((deprecated)) signature_t *signature_create(
    char *hash,
    list_t *shingles
);

void signature_free(signature_t *signature);

signature_t *signature_parseFromJSON(cJSON *signatureJSON);

cJSON *signature_convertToJSON(signature_t *signature);

#endif /* _signature_H_ */

