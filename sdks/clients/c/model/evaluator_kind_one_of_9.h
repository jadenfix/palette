/*
 * evaluator_kind_one_of_9.h
 *
 * Browser grounding: fraction of element-targeted steps that resolved to their intended element; score is the ratio, passes at &#x60;min_ratio&#x60;.
 */

#ifndef _evaluator_kind_one_of_9_H_
#define _evaluator_kind_one_of_9_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct evaluator_kind_one_of_9_t evaluator_kind_one_of_9_t;


// Enum TYPE for evaluator_kind_one_of_9

typedef enum  { beater_api_evaluator_kind_one_of_9_TYPE_NULL = 0, beater_api_evaluator_kind_one_of_9_TYPE_browser_grounding } beater_api_evaluator_kind_one_of_9_TYPE_e;

char* evaluator_kind_one_of_9_type_ToString(beater_api_evaluator_kind_one_of_9_TYPE_e type);

beater_api_evaluator_kind_one_of_9_TYPE_e evaluator_kind_one_of_9_type_FromString(char* type);



typedef struct evaluator_kind_one_of_9_t {
    double min_ratio; //numeric
    beater_api_evaluator_kind_one_of_9_TYPE_e type; //enum

    int _library_owned; // Is the library responsible for freeing this object?
} evaluator_kind_one_of_9_t;

__attribute__((deprecated)) evaluator_kind_one_of_9_t *evaluator_kind_one_of_9_create(
    double min_ratio,
    beater_api_evaluator_kind_one_of_9_TYPE_e type
);

void evaluator_kind_one_of_9_free(evaluator_kind_one_of_9_t *evaluator_kind_one_of_9);

evaluator_kind_one_of_9_t *evaluator_kind_one_of_9_parseFromJSON(cJSON *evaluator_kind_one_of_9JSON);

cJSON *evaluator_kind_one_of_9_convertToJSON(evaluator_kind_one_of_9_t *evaluator_kind_one_of_9);

#endif /* _evaluator_kind_one_of_9_H_ */

