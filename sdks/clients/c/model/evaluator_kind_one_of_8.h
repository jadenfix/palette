/*
 * evaluator_kind_one_of_8.h
 *
 * Browser step efficiency: passes when the run used at most &#x60;max_steps&#x60; browser steps (catches looping/backtracking). Reads &#x60;trace.browser_steps&#x60;.
 */

#ifndef _evaluator_kind_one_of_8_H_
#define _evaluator_kind_one_of_8_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct evaluator_kind_one_of_8_t evaluator_kind_one_of_8_t;


// Enum TYPE for evaluator_kind_one_of_8

typedef enum  { beater_api_evaluator_kind_one_of_8_TYPE_NULL = 0, beater_api_evaluator_kind_one_of_8_TYPE_browser_step_efficiency } beater_api_evaluator_kind_one_of_8_TYPE_e;

char* evaluator_kind_one_of_8_type_ToString(beater_api_evaluator_kind_one_of_8_TYPE_e type);

beater_api_evaluator_kind_one_of_8_TYPE_e evaluator_kind_one_of_8_type_FromString(char* type);



typedef struct evaluator_kind_one_of_8_t {
    long max_steps; //numeric
    beater_api_evaluator_kind_one_of_8_TYPE_e type; //enum

    int _library_owned; // Is the library responsible for freeing this object?
} evaluator_kind_one_of_8_t;

__attribute__((deprecated)) evaluator_kind_one_of_8_t *evaluator_kind_one_of_8_create(
    long max_steps,
    beater_api_evaluator_kind_one_of_8_TYPE_e type
);

void evaluator_kind_one_of_8_free(evaluator_kind_one_of_8_t *evaluator_kind_one_of_8);

evaluator_kind_one_of_8_t *evaluator_kind_one_of_8_parseFromJSON(cJSON *evaluator_kind_one_of_8JSON);

cJSON *evaluator_kind_one_of_8_convertToJSON(evaluator_kind_one_of_8_t *evaluator_kind_one_of_8);

#endif /* _evaluator_kind_one_of_8_H_ */

