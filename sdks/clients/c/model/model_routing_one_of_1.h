/*
 * model_routing_one_of_1.h
 *
 * Use the managed default model (hosted only).
 */

#ifndef _model_routing_one_of_1_H_
#define _model_routing_one_of_1_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct model_routing_one_of_1_t model_routing_one_of_1_t;

#include "model_ref.h"

// Enum KIND for model_routing_one_of_1

typedef enum  { beater_api_model_routing_one_of_1_KIND_NULL = 0, beater_api_model_routing_one_of_1_KIND_managed } beater_api_model_routing_one_of_1_KIND_e;

char* model_routing_one_of_1_kind_ToString(beater_api_model_routing_one_of_1_KIND_e kind);

beater_api_model_routing_one_of_1_KIND_e model_routing_one_of_1_kind_FromString(char* kind);



typedef struct model_routing_one_of_1_t {
    struct model_ref_t *default_model; //model
    beater_api_model_routing_one_of_1_KIND_e kind; //enum

    int _library_owned; // Is the library responsible for freeing this object?
} model_routing_one_of_1_t;

__attribute__((deprecated)) model_routing_one_of_1_t *model_routing_one_of_1_create(
    model_ref_t *default_model,
    beater_api_model_routing_one_of_1_KIND_e kind
);

void model_routing_one_of_1_free(model_routing_one_of_1_t *model_routing_one_of_1);

model_routing_one_of_1_t *model_routing_one_of_1_parseFromJSON(cJSON *model_routing_one_of_1JSON);

cJSON *model_routing_one_of_1_convertToJSON(model_routing_one_of_1_t *model_routing_one_of_1);

#endif /* _model_routing_one_of_1_H_ */

