/*
 * evaluator_kind_one_of_10.h
 *
 * Browser recovery: passes when the run either hit no errors or recovered to a successful final step (catches death spirals after a failed action).
 */

#ifndef _evaluator_kind_one_of_10_H_
#define _evaluator_kind_one_of_10_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct evaluator_kind_one_of_10_t evaluator_kind_one_of_10_t;


// Enum TYPE for evaluator_kind_one_of_10

typedef enum  { beater_api_evaluator_kind_one_of_10_TYPE_NULL = 0, beater_api_evaluator_kind_one_of_10_TYPE_browser_recovery } beater_api_evaluator_kind_one_of_10_TYPE_e;

char* evaluator_kind_one_of_10_type_ToString(beater_api_evaluator_kind_one_of_10_TYPE_e type);

beater_api_evaluator_kind_one_of_10_TYPE_e evaluator_kind_one_of_10_type_FromString(char* type);



typedef struct evaluator_kind_one_of_10_t {
    beater_api_evaluator_kind_one_of_10_TYPE_e type; //enum

    int _library_owned; // Is the library responsible for freeing this object?
} evaluator_kind_one_of_10_t;

__attribute__((deprecated)) evaluator_kind_one_of_10_t *evaluator_kind_one_of_10_create(
    beater_api_evaluator_kind_one_of_10_TYPE_e type
);

void evaluator_kind_one_of_10_free(evaluator_kind_one_of_10_t *evaluator_kind_one_of_10);

evaluator_kind_one_of_10_t *evaluator_kind_one_of_10_parseFromJSON(cJSON *evaluator_kind_one_of_10JSON);

cJSON *evaluator_kind_one_of_10_convertToJSON(evaluator_kind_one_of_10_t *evaluator_kind_one_of_10);

#endif /* _evaluator_kind_one_of_10_H_ */

