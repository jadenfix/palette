/*
 * model_routing.h
 *
 * Per-request credential routing policy.  This enum is the encoding of *\&quot;any model, optionally BYOK, else managed default\&quot;*:  * [&#x60;ModelRouting::Byok&#x60;] — use the tenant&#39;s own opaque provider secret. The   model named in the [&#x60;ChatCompletionRequest&#x60;] is used as-is, so any model the   key can reach is reachable. * [&#x60;ModelRouting::Managed&#x60;] — use Beater&#39;s managed credentials with the given   default model. Only available when the gateway is built with a   [&#x60;ManagedDefault&#x60;] (hosted edition).  A [&#x60;GatewayRequest&#x60;] carries an *ordered* &#x60;Vec&lt;ModelRouting&gt;&#x60; so the gateway can fail over from one key/provider to the next. An empty list means *\&quot;I did not bring a key\&quot;*: the gateway then uses the managed default if configured, or returns [&#x60;GatewayError::NoKeyAndNoManagedDefault&#x60;] in OSS.
 */

#ifndef _model_routing_H_
#define _model_routing_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct model_routing_t model_routing_t;

#include "model_ref.h"
#include "model_routing_one_of.h"
#include "model_routing_one_of_1.h"

// Enum KIND for model_routing

typedef enum  { beater_api_model_routing_KIND_NULL = 0, beater_api_model_routing_KIND_managed } beater_api_model_routing_KIND_e;

char* model_routing_kind_ToString(beater_api_model_routing_KIND_e kind);

beater_api_model_routing_KIND_e model_routing_kind_FromString(char* kind);



typedef struct model_routing_t {
    beater_api_model_routing_KIND_e kind; //enum
    char *provider_secret_id; // string
    struct model_ref_t *default_model; //model

    int _library_owned; // Is the library responsible for freeing this object?
} model_routing_t;

__attribute__((deprecated)) model_routing_t *model_routing_create(
    beater_api_model_routing_KIND_e kind,
    char *provider_secret_id,
    model_ref_t *default_model
);

void model_routing_free(model_routing_t *model_routing);

model_routing_t *model_routing_parseFromJSON(cJSON *model_routingJSON);

cJSON *model_routing_convertToJSON(model_routing_t *model_routing);

#endif /* _model_routing_H_ */

