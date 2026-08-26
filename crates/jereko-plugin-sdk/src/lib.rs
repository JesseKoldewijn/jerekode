//! Native plugin SDK — safe Rust helpers over `include/jereko_plugin.h`.

#![allow(clippy::missing_safety_doc)]

pub const ABI_VERSION: u32 = 1;

/// C-compatible plugin metadata (mirrors `JerekoPluginInfo`).
#[repr(C)]
pub struct PluginInfo {
    pub abi_version: u32,
    pub name: *const std::ffi::c_char,
    pub version: *const std::ffi::c_char,
}

/// C-compatible hook result (mirrors `JerekoHookResult`).
#[repr(C)]
pub struct HookResult {
    pub status: i32,
    pub json_output: *const std::ffi::c_char,
}

/// Parse a JSON payload from the host into `T`.
///
/// # Safety
/// `payload_json` must be a valid NUL-terminated C string pointer.
pub unsafe fn parse_payload<T: serde::de::DeserializeOwned>(
    payload_json: *const std::ffi::c_char,
) -> Result<T, String> {
    if payload_json.is_null() {
        return Err("null payload".into());
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(payload_json) };
    let text = cstr.to_str().map_err(|e| e.to_string())?;
    serde_json::from_str(text).map_err(|e| e.to_string())
}

/// Encode a JSON value as a NUL-terminated C string leaked for the host call.
///
/// The host must copy the bytes before the next invoke if it retains the pointer.
pub fn encode_output(value: &serde_json::Value) -> *const std::ffi::c_char {
    let text = value.to_string();
    let cstr = std::ffi::CString::new(text).unwrap_or_default();
    cstr.into_raw()
}

/// Export `jereko_plugin_info` / `jereko_plugin_invoke` for a plugin crate.
#[macro_export]
macro_rules! export_plugin {
    ($name:expr, $version:expr, $handler:expr) => {
        #[no_mangle]
        pub extern "C" fn jereko_plugin_info() -> $crate::PluginInfo {
            $crate::PluginInfo {
                abi_version: $crate::ABI_VERSION,
                name: concat!($name, "\0").as_ptr().cast(),
                version: concat!($version, "\0").as_ptr().cast(),
            }
        }

        #[no_mangle]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn jereko_plugin_invoke(
            hook: *const std::ffi::c_char,
            payload_json: *const std::ffi::c_char,
        ) -> $crate::HookResult {
            let hook_name = if hook.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(hook) }
                    .to_str()
                    .unwrap_or("")
            };
            let payload = if payload_json.is_null() {
                serde_json::json!({})
            } else {
                match unsafe { std::ffi::CStr::from_ptr(payload_json) }.to_str() {
                    Ok(text) => serde_json::from_str(text).unwrap_or(serde_json::json!({})),
                    Err(_) => serde_json::json!({}),
                }
            };
            let handler: fn(&str, serde_json::Value) -> Result<serde_json::Value, i32> = $handler;
            match handler(hook_name, payload) {
                Ok(value) => $crate::HookResult {
                    status: 0,
                    json_output: $crate::encode_output(&value),
                },
                Err(status) => $crate::HookResult {
                    status,
                    json_output: std::ptr::null(),
                },
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_round_trips_payload_helpers() {
        let value = serde_json::json!({"ok": true});
        let ptr = encode_output(&value);
        assert!(!ptr.is_null());
        let parsed: serde_json::Value = unsafe { parse_payload(ptr) }.unwrap();
        assert_eq!(parsed["ok"], true);
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr as *mut std::ffi::c_char);
        }
    }
}
