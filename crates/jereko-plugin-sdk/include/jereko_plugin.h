#ifndef JEREKO_PLUGIN_H
#define JEREKO_PLUGIN_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define JEREKO_PLUGIN_ABI_VERSION 1

typedef struct JerekoPluginInfo {
    uint32_t abi_version;
    const char *name;
    const char *version;
} JerekoPluginInfo;

typedef struct JerekoHookResult {
    int32_t status;
    const char *json_output;
} JerekoHookResult;

typedef JerekoPluginInfo (*jereko_plugin_info_fn)(void);
typedef JerekoHookResult (*jereko_plugin_invoke_fn)(const char *hook, const char *payload_json);

#ifdef __cplusplus
}
#endif

#endif /* JEREKO_PLUGIN_H */
