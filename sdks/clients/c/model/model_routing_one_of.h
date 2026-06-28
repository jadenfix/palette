/*
 * model_routing_one_of.h
 *
 * Bring-your-own-key: resolve the tenant&#39;s opaque provider secret.
 */

#ifndef _model_routing_one_of_H_
#define _model_routing_one_of_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct model_routing_one_of_t model_routing_one_of_t;


// Enum KIND for model_routing_one_of

typedef enum  { beater_api_model_routing_one_of_KIND_NULL = 0, beater_api_model_routing_one_of_KIND_byok } beater_api_model_routing_one_of_KIND_e;

char* model_routing_one_of_kind_ToString(beater_api_model_routing_one_of_KIND_e kind);

beater_api_model_routing_one_of_KIND_e model_routing_one_of_kind_FromString(char* kind);



typedef struct model_routing_one_of_t {
    beater_api_model_routing_one_of_KIND_e kind; //enum
    char *provider_secret_id; // string

    int _library_owned; // Is the library responsible for freeing this object?
} model_routing_one_of_t;

__attribute__((deprecated)) model_routing_one_of_t *model_routing_one_of_create(
    beater_api_model_routing_one_of_KIND_e kind,
    char *provider_secret_id
);

void model_routing_one_of_free(model_routing_one_of_t *model_routing_one_of);

model_routing_one_of_t *model_routing_one_of_parseFromJSON(cJSON *model_routing_one_ofJSON);

cJSON *model_routing_one_of_convertToJSON(model_routing_one_of_t *model_routing_one_of);

#endif /* _model_routing_one_of_H_ */

