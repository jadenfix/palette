#include <stdlib.h>
#include <stdio.h>
#include "../include/apiClient.h"
#include "../include/list.h"
#include "../external/cJSON.h"
#include "../include/keyValuePair.h"
#include "../include/binary.h"
#include "../model/create_review_queue_http_request.h"
#include "../model/dataset_case.h"
#include "../model/enqueue_review_task_from_trace_http_request.h"
#include "../model/error_response.h"
#include "../model/promote_review_annotation_http_request.h"
#include "../model/review_annotation.h"
#include "../model/review_queue.h"
#include "../model/review_task.h"
#include "../model/review_task_state.h"
#include "../model/submit_review_annotation_http_request.h"

// Enum  for ReviewsAPI_reviewsListReviewTasks
typedef enum  { beater_api_reviewsListReviewTasks__NULL = 0, beater_api_reviewsListReviewTasks__open, beater_api_reviewsListReviewTasks__submitted, beater_api_reviewsListReviewTasks__cancelled } beater_api_reviewsListReviewTasks_state_e;


review_queue_t*
ReviewsAPI_reviewsCreateReviewQueue(apiClient_t *apiClient, char *tenant_id, char *project_id, create_review_queue_http_request_t *create_review_queue_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


review_task_t*
ReviewsAPI_reviewsEnqueueReviewTaskFromTrace(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, enqueue_review_task_from_trace_http_request_t *enqueue_review_task_from_trace_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


list_t*
ReviewsAPI_reviewsListReviewTasks(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, review_task_state_e state, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


dataset_case_t*
ReviewsAPI_reviewsPromoteReviewAnnotation(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, char *task_id, char *annotation_id, promote_review_annotation_http_request_t *promote_review_annotation_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


review_annotation_t*
ReviewsAPI_reviewsSubmitReviewAnnotation(apiClient_t *apiClient, char *tenant_id, char *project_id, char *queue_id, char *task_id, submit_review_annotation_http_request_t *submit_review_annotation_http_request, char *authorization, char *x_beater_api_key, char *x_beater_project_id, char *x_beater_environment_id);


