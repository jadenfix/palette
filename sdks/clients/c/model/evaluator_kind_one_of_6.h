/*
 * evaluator_kind_one_of_6.h
 *
 * 
 */

#ifndef _evaluator_kind_one_of_6_H_
#define _evaluator_kind_one_of_6_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct evaluator_kind_one_of_6_t evaluator_kind_one_of_6_t;


// Enum TYPE for evaluator_kind_one_of_6

typedef enum  { beater_api_evaluator_kind_one_of_6_TYPE_NULL = 0, beater_api_evaluator_kind_one_of_6_TYPE_llm_judge } beater_api_evaluator_kind_one_of_6_TYPE_e;

char* evaluator_kind_one_of_6_type_ToString(beater_api_evaluator_kind_one_of_6_TYPE_e type);

beater_api_evaluator_kind_one_of_6_TYPE_e evaluator_kind_one_of_6_type_FromString(char* type);



typedef struct evaluator_kind_one_of_6_t {
    char *model; // string
    char *rubric; // string
    beater_api_evaluator_kind_one_of_6_TYPE_e type; //enum

    int _library_owned; // Is the library responsible for freeing this object?
} evaluator_kind_one_of_6_t;

__attribute__((deprecated)) evaluator_kind_one_of_6_t *evaluator_kind_one_of_6_create(
    char *model,
    char *rubric,
    beater_api_evaluator_kind_one_of_6_TYPE_e type
);

void evaluator_kind_one_of_6_free(evaluator_kind_one_of_6_t *evaluator_kind_one_of_6);

evaluator_kind_one_of_6_t *evaluator_kind_one_of_6_parseFromJSON(cJSON *evaluator_kind_one_of_6JSON);

cJSON *evaluator_kind_one_of_6_convertToJSON(evaluator_kind_one_of_6_t *evaluator_kind_one_of_6);

#endif /* _evaluator_kind_one_of_6_H_ */

