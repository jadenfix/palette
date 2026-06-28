/*
 * evaluator_kind_one_of_3.h
 *
 * 
 */

#ifndef _evaluator_kind_one_of_3_H_
#define _evaluator_kind_one_of_3_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct evaluator_kind_one_of_3_t evaluator_kind_one_of_3_t;


// Enum TYPE for evaluator_kind_one_of_3

typedef enum  { beater_api_evaluator_kind_one_of_3_TYPE_NULL = 0, beater_api_evaluator_kind_one_of_3_TYPE_json_object } beater_api_evaluator_kind_one_of_3_TYPE_e;

char* evaluator_kind_one_of_3_type_ToString(beater_api_evaluator_kind_one_of_3_TYPE_e type);

beater_api_evaluator_kind_one_of_3_TYPE_e evaluator_kind_one_of_3_type_FromString(char* type);



typedef struct evaluator_kind_one_of_3_t {
    beater_api_evaluator_kind_one_of_3_TYPE_e type; //enum

    int _library_owned; // Is the library responsible for freeing this object?
} evaluator_kind_one_of_3_t;

__attribute__((deprecated)) evaluator_kind_one_of_3_t *evaluator_kind_one_of_3_create(
    beater_api_evaluator_kind_one_of_3_TYPE_e type
);

void evaluator_kind_one_of_3_free(evaluator_kind_one_of_3_t *evaluator_kind_one_of_3);

evaluator_kind_one_of_3_t *evaluator_kind_one_of_3_parseFromJSON(cJSON *evaluator_kind_one_of_3JSON);

cJSON *evaluator_kind_one_of_3_convertToJSON(evaluator_kind_one_of_3_t *evaluator_kind_one_of_3);

#endif /* _evaluator_kind_one_of_3_H_ */

