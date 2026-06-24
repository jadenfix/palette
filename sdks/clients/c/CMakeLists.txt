cmake_minimum_required (VERSION 2.6...3.10.2)
project (CGenerator C)

cmake_policy(SET CMP0063 NEW)

set(CMAKE_C_VISIBILITY_PRESET default)
set(CMAKE_VISIBILITY_INLINES_HIDDEN OFF)
set(CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS ON)
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Werror=implicit-function-declaration")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Werror=missing-declarations")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Werror=int-conversion")

option(BUILD_SHARED_LIBS "Build using shared libraries" ON)

find_package(OpenSSL)

if (OPENSSL_FOUND)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -DOPENSSL")
    if(CMAKE_VERSION VERSION_LESS 3.4)
        include_directories(${OPENSSL_INCLUDE_DIR})
        include_directories(${OPENSSL_INCLUDE_DIRS})
        link_directories(${OPENSSL_LIBRARIES})
    endif()
endif()

set(pkgName "beater_api")

# this default version can be overridden in PreTarget.cmake
set(PROJECT_VERSION_MAJOR 0)
set(PROJECT_VERSION_MINOR 0)
set(PROJECT_VERSION_PATCH 1)

if( (DEFINED CURL_INCLUDE_DIR) AND (DEFINED CURL_LIBRARIES))
    include_directories(${CURL_INCLUDE_DIR})
    set(PLATFORM_LIBRARIES ${PLATFORM_LIBRARIES} ${CURL_LIBRARIES} )
else()
    find_package(CURL 7.58.0 REQUIRED)
    if(CURL_FOUND)
        include_directories(${CURL_INCLUDE_DIR})
        set(PLATFORM_LIBRARIES ${PLATFORM_LIBRARIES} ${CURL_LIBRARIES} )
    endif()
endif()

set(SRCS
    src/list.c
    src/apiKey.c
    src/apiClient.c
    src/binary.c
    external/cJSON.c
    model/object.c
    model/alert_decision.c
    model/alert_input.c
    model/alert_links.c
    model/alert_policy.c
    model/alert_severity.c
    model/api_key_created_response.c
    model/api_scope.c
    model/archive_manifest.c
    model/archive_query_response.c
    model/archived_span_row.c
    model/artifact_ref.c
    model/audit_action.c
    model/audit_event.c
    model/audit_outcome.c
    model/auth_context.c
    model/bus_message.c
    model/calibration_confusion.c
    model/calibration_item.c
    model/calibration_label.c
    model/calibration_policy.c
    model/calibration_report.c
    model/canonical_span.c
    model/case_experiment_score.c
    model/case_output_override_request.c
    model/create_api_key_http_request.c
    model/create_dataset_request.c
    model/create_dataset_version_request.c
    model/create_gate_request.c
    model/create_provider_secret_http_request.c
    model/create_review_queue_http_request.c
    model/currency.c
    model/dataset.c
    model/dataset_case.c
    model/dataset_eval_report.c
    model/dataset_version_snapshot.c
    model/dead_letter.c
    model/dead_letter_replay_report.c
    model/enqueue_review_task_from_trace_http_request.c
    model/error_response.c
    model/eval_reproducibility.c
    model/eval_result.c
    model/evaluate_alert_request.c
    model/evaluation_case.c
    model/evaluator_kind.c
    model/evaluator_kind_one_of.c
    model/evaluator_kind_one_of_1.c
    model/evaluator_kind_one_of_2.c
    model/evaluator_kind_one_of_3.c
    model/evaluator_kind_one_of_4.c
    model/evaluator_kind_one_of_5.c
    model/evaluator_lane.c
    model/evaluator_spec.c
    model/experiment_comparison.c
    model/experiment_run_report.c
    model/gate_decision.c
    model/gate_definition.c
    model/gate_policy.c
    model/gate_run_report.c
    model/health_response.c
    model/import_source_http_request.c
    model/inconclusive_policy.c
    model/ingest_outcome.c
    model/ingest_queue_status.c
    model/judge_audit_record.c
    model/judge_broker_outcome.c
    model/maintenance_window.c
    model/model_ref.c
    model/money.c
    model/native_ingest_request.c
    model/online_sampling_policy.c
    model/otlp_ingest_outcome.c
    model/page_run_summary.c
    model/page_run_summary_items_inner.c
    model/promote_review_annotation_http_request.c
    model/promote_trace_case_request.c
    model/provider_secret_metadata.c
    model/publish_ack.c
    model/queued_trace_work.c
    model/redaction_class.c
    model/review_annotation.c
    model/review_queue.c
    model/review_task.c
    model/review_task_state.c
    model/review_verdict.c
    model/revoked_api_key.c
    model/revoked_provider_secret.c
    model/run_calibration_http_request.c
    model/run_deterministic_eval_request.c
    model/run_experiment_request.c
    model/run_gate_request.c
    model/run_judge_dataset_eval_request.c
    model/run_judge_eval_http_request.c
    model/run_judge_experiment_request.c
    model/run_summary.c
    model/sampling_decision.c
    model/sampling_reason.c
    model/score_result.c
    model/search_hit.c
    model/search_response.c
    model/span_io_response.c
    model/span_io_value.c
    model/span_io_value_one_of.c
    model/span_io_value_one_of_1.c
    model/span_io_value_one_of_2.c
    model/span_io_value_one_of_3.c
    model/span_status.c
    model/statistical_test.c
    model/submit_review_annotation_http_request.c
    model/tenant_scope.c
    model/token_counts.c
    model/trace_ingested_drain_report.c
    model/trace_ingested_reconcile_report.c
    model/trace_view.c
    model/trace_write_drain_report.c
    model/usage_summary.c
    model/usage_total.c
    model/webhook_delivery.c
    model/write_ack.c
    api/AlertsAPI.c
    api/ApiKeysAPI.c
    api/ArchiveAPI.c
    api/AuditAPI.c
    api/CalibrationsAPI.c
    api/DatasetsAPI.c
    api/EvalsAPI.c
    api/ExperimentsAPI.c
    api/GatesAPI.c
    api/HealthAPI.c
    api/IngestAPI.c
    api/JudgeAPI.c
    api/OnlineAPI.c
    api/ProviderSecretsAPI.c
    api/ReviewsAPI.c
    api/SearchAPI.c
    api/SpansAPI.c
    api/TracesAPI.c
    api/UsageAPI.c

)

set(HDRS
    include/apiClient.h
    include/list.h
    include/binary.h
    include/keyValuePair.h
    external/cJSON.h
    model/object.h
    model/any_type.h
    model/alert_decision.h
    model/alert_input.h
    model/alert_links.h
    model/alert_policy.h
    model/alert_severity.h
    model/api_key_created_response.h
    model/api_scope.h
    model/archive_manifest.h
    model/archive_query_response.h
    model/archived_span_row.h
    model/artifact_ref.h
    model/audit_action.h
    model/audit_event.h
    model/audit_outcome.h
    model/auth_context.h
    model/bus_message.h
    model/calibration_confusion.h
    model/calibration_item.h
    model/calibration_label.h
    model/calibration_policy.h
    model/calibration_report.h
    model/canonical_span.h
    model/case_experiment_score.h
    model/case_output_override_request.h
    model/create_api_key_http_request.h
    model/create_dataset_request.h
    model/create_dataset_version_request.h
    model/create_gate_request.h
    model/create_provider_secret_http_request.h
    model/create_review_queue_http_request.h
    model/currency.h
    model/dataset.h
    model/dataset_case.h
    model/dataset_eval_report.h
    model/dataset_version_snapshot.h
    model/dead_letter.h
    model/dead_letter_replay_report.h
    model/enqueue_review_task_from_trace_http_request.h
    model/error_response.h
    model/eval_reproducibility.h
    model/eval_result.h
    model/evaluate_alert_request.h
    model/evaluation_case.h
    model/evaluator_kind.h
    model/evaluator_kind_one_of.h
    model/evaluator_kind_one_of_1.h
    model/evaluator_kind_one_of_2.h
    model/evaluator_kind_one_of_3.h
    model/evaluator_kind_one_of_4.h
    model/evaluator_kind_one_of_5.h
    model/evaluator_lane.h
    model/evaluator_spec.h
    model/experiment_comparison.h
    model/experiment_run_report.h
    model/gate_decision.h
    model/gate_definition.h
    model/gate_policy.h
    model/gate_run_report.h
    model/health_response.h
    model/import_source_http_request.h
    model/inconclusive_policy.h
    model/ingest_outcome.h
    model/ingest_queue_status.h
    model/judge_audit_record.h
    model/judge_broker_outcome.h
    model/maintenance_window.h
    model/model_ref.h
    model/money.h
    model/native_ingest_request.h
    model/online_sampling_policy.h
    model/otlp_ingest_outcome.h
    model/page_run_summary.h
    model/page_run_summary_items_inner.h
    model/promote_review_annotation_http_request.h
    model/promote_trace_case_request.h
    model/provider_secret_metadata.h
    model/publish_ack.h
    model/queued_trace_work.h
    model/redaction_class.h
    model/review_annotation.h
    model/review_queue.h
    model/review_task.h
    model/review_task_state.h
    model/review_verdict.h
    model/revoked_api_key.h
    model/revoked_provider_secret.h
    model/run_calibration_http_request.h
    model/run_deterministic_eval_request.h
    model/run_experiment_request.h
    model/run_gate_request.h
    model/run_judge_dataset_eval_request.h
    model/run_judge_eval_http_request.h
    model/run_judge_experiment_request.h
    model/run_summary.h
    model/sampling_decision.h
    model/sampling_reason.h
    model/score_result.h
    model/search_hit.h
    model/search_response.h
    model/span_io_response.h
    model/span_io_value.h
    model/span_io_value_one_of.h
    model/span_io_value_one_of_1.h
    model/span_io_value_one_of_2.h
    model/span_io_value_one_of_3.h
    model/span_status.h
    model/statistical_test.h
    model/submit_review_annotation_http_request.h
    model/tenant_scope.h
    model/token_counts.h
    model/trace_ingested_drain_report.h
    model/trace_ingested_reconcile_report.h
    model/trace_view.h
    model/trace_write_drain_report.h
    model/usage_summary.h
    model/usage_total.h
    model/webhook_delivery.h
    model/write_ack.h
    api/AlertsAPI.h
    api/ApiKeysAPI.h
    api/ArchiveAPI.h
    api/AuditAPI.h
    api/CalibrationsAPI.h
    api/DatasetsAPI.h
    api/EvalsAPI.h
    api/ExperimentsAPI.h
    api/GatesAPI.h
    api/HealthAPI.h
    api/IngestAPI.h
    api/JudgeAPI.h
    api/OnlineAPI.h
    api/ProviderSecretsAPI.h
    api/ReviewsAPI.h
    api/SearchAPI.h
    api/SpansAPI.h
    api/TracesAPI.h
    api/UsageAPI.h

)

include(PreTarget.cmake OPTIONAL)

set(PROJECT_VERSION_STRING "${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}.${PROJECT_VERSION_PATCH}")

# Add library with project file with project name as library name
add_library(${pkgName} ${SRCS} ${HDRS})
# Link dependent libraries
if(NOT CMAKE_VERSION VERSION_LESS 3.4)
    target_link_libraries(${pkgName} PRIVATE OpenSSL::SSL OpenSSL::Crypto)
endif()
target_link_libraries(${pkgName} PUBLIC ${CURL_LIBRARIES} )
target_include_directories(
    ${pkgName} PUBLIC $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}>
    $<INSTALL_INTERFACE:include>
)

include(PostTarget.cmake OPTIONAL)

# installation of libraries, headers, and config files
if (NOT EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/Config.cmake.in)
    install(TARGETS ${pkgName} DESTINATION lib)
else()
    include(GNUInstallDirs)
    install(TARGETS ${pkgName} DESTINATION lib EXPORT ${pkgName}Targets)

    foreach(HDR_FILE ${HDRS})
        get_filename_component(HDR_DIRECTORY ${HDR_FILE} DIRECTORY)
        get_filename_component(ABSOLUTE_HDR_DIRECTORY ${HDR_DIRECTORY} ABSOLUTE)
        file(RELATIVE_PATH RELATIVE_HDR_PATH ${CMAKE_CURRENT_SOURCE_DIR} ${ABSOLUTE_HDR_DIRECTORY})
        install(FILES ${HDR_FILE} DESTINATION include/${pkgName}/${RELATIVE_HDR_PATH})
    endforeach()

    include(CMakePackageConfigHelpers)
    write_basic_package_version_file(
            "${CMAKE_CURRENT_BINARY_DIR}/${pkgName}/${pkgName}ConfigVersion.cmake"
            VERSION "${PROJECT_VERSION_STRING}"
            COMPATIBILITY AnyNewerVersion
    )

    export(EXPORT ${pkgName}Targets
            FILE "${CMAKE_CURRENT_BINARY_DIR}/${pkgName}/${pkgName}Targets.cmake"
            NAMESPACE ${pkgName}::
            )

    configure_file(${CMAKE_CURRENT_SOURCE_DIR}/Config.cmake.in
            "${CMAKE_CURRENT_BINARY_DIR}/${pkgName}/${pkgName}Config.cmake"
            @ONLY
            )

    set(ConfigPackageLocation lib/cmake/${pkgName})
    install(EXPORT ${pkgName}Targets
            FILE
            ${pkgName}Targets.cmake
            NAMESPACE
            ${pkgName}::
            DESTINATION
            ${ConfigPackageLocation}
            )
    install(
            FILES
            "${CMAKE_CURRENT_BINARY_DIR}/${pkgName}/${pkgName}Config.cmake"
            "${CMAKE_CURRENT_BINARY_DIR}/${pkgName}/${pkgName}ConfigVersion.cmake"
            DESTINATION
            ${ConfigPackageLocation}
    )
endif()

# make installation packages
include(Packing.cmake OPTIONAL)

# Setting file variables to null
set(SRCS "")
set(HDRS "")


## This section shows how to use the above compiled library to compile the source files
## set source files
#set(SRCS
#    unit-tests/manual-AlertsAPI.c
#    unit-tests/manual-ApiKeysAPI.c
#    unit-tests/manual-ArchiveAPI.c
#    unit-tests/manual-AuditAPI.c
#    unit-tests/manual-CalibrationsAPI.c
#    unit-tests/manual-DatasetsAPI.c
#    unit-tests/manual-EvalsAPI.c
#    unit-tests/manual-ExperimentsAPI.c
#    unit-tests/manual-GatesAPI.c
#    unit-tests/manual-HealthAPI.c
#    unit-tests/manual-IngestAPI.c
#    unit-tests/manual-JudgeAPI.c
#    unit-tests/manual-OnlineAPI.c
#    unit-tests/manual-ProviderSecretsAPI.c
#    unit-tests/manual-ReviewsAPI.c
#    unit-tests/manual-SearchAPI.c
#    unit-tests/manual-SpansAPI.c
#    unit-tests/manual-TracesAPI.c
#    unit-tests/manual-UsageAPI.c
#)

##set header files
#set(HDRS
#)

## loop over all files in SRCS variable
#foreach(SOURCE_FILE ${SRCS})
#    # Get only the file name from the file as add_executable does not support executable with slash("/")
#    get_filename_component(FILE_NAME_ONLY ${SOURCE_FILE} NAME_WE)
#    # Remove .c from the file name and set it as executable name
#    string( REPLACE ".c" "" EXECUTABLE_FILE ${FILE_NAME_ONLY})
#    # Add executable for every source file in  SRCS
#    add_executable(unit-${EXECUTABLE_FILE} ${SOURCE_FILE})
#    # Link above created library to executable and dependent library curl
#    target_link_libraries(unit-${EXECUTABLE_FILE} ${CURL_LIBRARIES} ${pkgName} )
#endforeach(SOURCE_FILE ${SRCS})
