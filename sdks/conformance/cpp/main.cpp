// Live conformance: drive the GENERATED C++ (cpp-restsdk) control-plane client
// against a running beaterd and verify typed request/response shapes.
// Proves API==SDK for C++.
#include <cstdlib>
#include <iostream>
#include <memory>
#include <string>

#include "beater-client/ApiClient.h"
#include "beater-client/ApiConfiguration.h"
#include "beater-client/api/HealthApi.h"
#include "beater-client/api/DatasetsApi.h"
#include "beater-client/api/TracesApi.h"
#include "beater-client/model/CreateDatasetRequest.h"

using namespace org::openapitools::client;

static std::string env_or(const char* key, const char* def) {
    const char* v = std::getenv(key);
    return (v && *v) ? std::string(v) : std::string(def);
}

int main() {
    const char* base = std::getenv("BEATER_BASE_URL");
    if (!base || !*base) {
        std::cerr << "FAIL: BEATER_BASE_URL unset\n";
        return 1;
    }
    std::string tenant = env_or("BEATER_TENANT", "demo");
    std::string project = env_or("BEATER_PROJECT", "demo");

    auto config = std::make_shared<api::ApiConfiguration>();
    config->setBaseUrl(utility::conversions::to_string_t(std::string(base)));
    auto apiClient = std::make_shared<api::ApiClient>(config);

    try {
        // GET /health -> typed HealthResponse
        api::HealthApi healthApi(apiClient);
        auto health = healthApi.health_health().get();
        if (!health->isOk()) {
            std::cerr << "FAIL: health ok != true\n";
            return 1;
        }
        std::cout << "  health ok=" << std::boolalpha << health->isOk() << "\n";

        // POST /v1/datasets/{tenant}/{project} -> typed Dataset
        api::DatasetsApi datasetsApi(apiClient);
        auto req = std::make_shared<model::CreateDatasetRequest>();
        req->setName(utility::conversions::to_string_t(std::string("conformance-cpp")));
        auto ds = datasetsApi.datasets_createDataset(
                      utility::conversions::to_string_t(tenant),
                      utility::conversions::to_string_t(project),
                      req, boost::none, boost::none, boost::none, boost::none)
                      .get();
        std::cout << "  createDataset -> ok ("
                  << utility::conversions::to_utf8string(ds->getName()) << ")\n";

        // GET /v1/traces/{tenant} -> typed Page_RunSummary
        api::TracesApi tracesApi(apiClient);
        auto page = tracesApi.traces_listTraces(
                        utility::conversions::to_string_t(tenant),
                        boost::none, boost::none, boost::none, boost::none,
                        boost::none, boost::none, boost::none, boost::none,
                        boost::none, boost::none, boost::none, boost::none,
                        boost::none, boost::none, boost::none, boost::none,
                        boost::none, boost::none, boost::none)
                        .get();
        std::cout << "  traces.list items=" << page->getItems().size() << "\n";

        std::cout << "PASS: cpp generated client round-trips against live API\n";
        return 0;
    } catch (const api::ApiException& e) {
        std::cerr << "FAIL: ApiException: " << e.what() << "\n";
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "FAIL: " << e.what() << "\n";
        return 1;
    }
}
