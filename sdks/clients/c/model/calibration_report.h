/*
 * calibration_report.h
 *
 * 
 */

#ifndef _calibration_report_H_
#define _calibration_report_H_

#include <string.h>
#include "../external/cJSON.h"
#include "../include/list.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"

typedef struct calibration_report_t calibration_report_t;

#include "calibration_confusion.h"
#include "calibration_item.h"
#include "calibration_policy.h"
#include "reliability_bin.h"



typedef struct calibration_report_t {
    double brier_score; //numeric
    char *calibration_report_id; // string
    double cohen_kappa; //numeric
    double cohen_kappa_ci_high; //numeric
    double cohen_kappa_ci_low; //numeric
    struct calibration_confusion_t *confusion; //model
    char *created_at; //date time
    char *dataset_id; // string
    char *dataset_version_id; // string
    char *eval_report_id; // string
    char *evaluator_version_id; // string
    double expected_agreement; //numeric
    double expected_calibration_error; //numeric
    list_t *items; //nonprimitive container
    double observed_agreement; //numeric
    double observed_agreement_ci_high; //numeric
    double observed_agreement_ci_low; //numeric
    struct calibration_policy_t *policy; //model
    char *project_id; // string
    list_t *reliability_bins; //nonprimitive container
    int sample_count; //numeric
    char *tenant_id; // string

    int _library_owned; // Is the library responsible for freeing this object?
} calibration_report_t;

__attribute__((deprecated)) calibration_report_t *calibration_report_create(
    double brier_score,
    char *calibration_report_id,
    double cohen_kappa,
    double cohen_kappa_ci_high,
    double cohen_kappa_ci_low,
    calibration_confusion_t *confusion,
    char *created_at,
    char *dataset_id,
    char *dataset_version_id,
    char *eval_report_id,
    char *evaluator_version_id,
    double expected_agreement,
    double expected_calibration_error,
    list_t *items,
    double observed_agreement,
    double observed_agreement_ci_high,
    double observed_agreement_ci_low,
    calibration_policy_t *policy,
    char *project_id,
    list_t *reliability_bins,
    int sample_count,
    char *tenant_id
);

void calibration_report_free(calibration_report_t *calibration_report);

calibration_report_t *calibration_report_parseFromJSON(cJSON *calibration_reportJSON);

cJSON *calibration_report_convertToJSON(calibration_report_t *calibration_report);

#endif /* _calibration_report_H_ */

