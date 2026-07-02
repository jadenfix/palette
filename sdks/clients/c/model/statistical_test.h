/*
 * statistical_test.h
 *
 * The statistical test that produced an [&#x60;ExperimentComparison&#x60;]. The gate records which method was **actually executed** so a reader can tell a t-test result from an exact McNemar, Wilcoxon, bootstrap, cluster-robust, or anytime-valid sequential one. The old single &#x60;PairedNormalApproximation&#x60; (a hard-coded-z normal approximation with no p-value) is gone — see &#x60;beater-stats&#x60;.
 */

#ifndef _statistical_test_H_
#define _statistical_test_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct statistical_test_t statistical_test_t;


// Enum  for statistical_test

typedef enum { beater_api_statistical_test__NULL = 0, beater_api_statistical_test__paired_t, beater_api_statistical_test__mcnemar_exact, beater_api_statistical_test__wilcoxon_signed_rank, beater_api_statistical_test__paired_bootstrap, beater_api_statistical_test__clustered_paired_t, beater_api_statistical_test__sequential_e_value } beater_api_statistical_test__e;

char* statistical_test_statistical_test_ToString(beater_api_statistical_test__e statistical_test);

beater_api_statistical_test__e statistical_test_statistical_test_FromString(char* statistical_test);

cJSON *statistical_test_convertToJSON(beater_api_statistical_test__e statistical_test);

beater_api_statistical_test__e statistical_test_parseFromJSON(cJSON *statistical_testJSON);

#endif /* _statistical_test_H_ */

