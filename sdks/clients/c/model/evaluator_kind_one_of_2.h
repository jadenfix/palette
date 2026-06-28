/*
 * evaluator_kind_one_of_2.h
 *
 * 
 */

#ifndef _evaluator_kind_one_of_2_H_
#define _evaluator_kind_one_of_2_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct evaluator_kind_one_of_2_t evaluator_kind_one_of_2_t;


// Enum TYPE for evaluator_kind_one_of_2

typedef enum  { beater_api_evaluator_kind_one_of_2_TYPE_NULL = 0, beater_api_evaluator_kind_one_of_2_TYPE_numeric_tolerance } beater_api_evaluator_kind_one_of_2_TYPE_e;

char* evaluator_kind_one_of_2_type_ToString(beater_api_evaluator_kind_one_of_2_TYPE_e type);

beater_api_evaluator_kind_one_of_2_TYPE_e evaluator_kind_one_of_2_type_FromString(char* type);



typedef struct evaluator_kind_one_of_2_t {
    double abs; //numeric
    double rel; //numeric
    beater_api_evaluator_kind_one_of_2_TYPE_e type; //enum

    int _library_owned; // Is the library responsible for freeing this object?
} evaluator_kind_one_of_2_t;

__attribute__((deprecated)) evaluator_kind_one_of_2_t *evaluator_kind_one_of_2_create(
    double abs,
    double rel,
    beater_api_evaluator_kind_one_of_2_TYPE_e type
);

void evaluator_kind_one_of_2_free(evaluator_kind_one_of_2_t *evaluator_kind_one_of_2);

evaluator_kind_one_of_2_t *evaluator_kind_one_of_2_parseFromJSON(cJSON *evaluator_kind_one_of_2JSON);

cJSON *evaluator_kind_one_of_2_convertToJSON(evaluator_kind_one_of_2_t *evaluator_kind_one_of_2);

#endif /* _evaluator_kind_one_of_2_H_ */

