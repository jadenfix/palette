#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/calibration_report.h"
#include "../model/error_response.h"
#include "../model/run_calibration_http_request.h"


calibration_report_t*
CalibrationsAPI_calibrationsRunCalibration(apiClient_t *apiClient, char *tenant_id, char *project_id, char *dataset_id, char *version_id, run_calibration_http_request_t *run_calibration_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


