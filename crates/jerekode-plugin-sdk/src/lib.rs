//! Native plugin SDK — safe Rust helpers over `include/jerekode_plugin.h`.

#![allow(clippy::missing_safety_doc)]

pub const ABI_VERSION: u32 = 1;

/// C-compatible plugin metadata (mirrors `JerekodePluginInfo`).
#[repr(C)]
pub struct PluginInfo {
    pub abi_version: u32,
    pub name: *const std::ffi::c_char,
    pub version: *const std::ffi::c_char,
}

/// C-compatible hook result (mirrors `JerekodeHookResult`).
#[repr(C)]
pub struct HookResult {
    pub status: i32,
    pub json_output: *const std::ffi::c_char,
}

/// Parse a JSON payload from the host into `T`.
///
/// # Safety
/// `payload_json` must be a valid NUL-terminated C string pointer, or null.
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

/// Build plugin metadata for `jerekode_plugin_info` exports.
pub fn make_plugin_info(
    name: *const std::ffi::c_char,
    version: *const std::ffi::c_char,
) -> PluginInfo {
    PluginInfo {
        abi_version: ABI_VERSION,
        name,
        version,
    }
}

/// Dispatch a hook for `jerekode_plugin_invoke` exports.
///
/// # Safety
/// `hook` and `payload_json` must be null or valid NUL-terminated C strings.
pub unsafe fn dispatch_invoke(
    hook: *const std::ffi::c_char,
    payload_json: *const std::ffi::c_char,
    handler: fn(&str, serde_json::Value) -> Result<serde_json::Value, i32>,
) -> HookResult {
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
    match handler(hook_name, payload) {
        Ok(value) => HookResult {
            status: 0,
            json_output: encode_output(&value),
        },
        Err(status) => HookResult {
            status,
            json_output: std::ptr::null(),
        },
    }
}

/// Export `jerekode_plugin_info` / `jerekode_plugin_invoke` for a plugin crate.
#[macro_export]
macro_rules! export_plugin {
    ($name:expr, $version:expr, $handler:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn jerekode_plugin_info() -> $crate::PluginInfo {
            $crate::make_plugin_info(
                concat!($name, "\0").as_ptr().cast(),
                concat!($version, "\0").as_ptr().cast(),
            )
        }

        #[unsafe(no_mangle)]
        #[allow(clippy::not_unsafe_ptr_arg_deref)]
        pub extern "C" fn jerekode_plugin_invoke(
            hook: *const std::ffi::c_char,
            payload_json: *const std::ffi::c_char,
        ) -> $crate::HookResult {
            unsafe { $crate::dispatch_invoke(hook, payload_json, $handler) }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

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

    #[test]
    fn parse_payload_rejects_null() {
        let err = unsafe { parse_payload::<serde_json::Value>(std::ptr::null()) }.unwrap_err();
        assert_eq!(err, "null payload");
    }

    #[test]
    fn make_plugin_info_sets_abi() {
        let name = CString::new("demo").unwrap();
        let version = CString::new("1.2.3").unwrap();
        let info = make_plugin_info(name.as_ptr(), version.as_ptr());
        assert_eq!(info.abi_version, ABI_VERSION);
        assert_eq!(unsafe { std::ffi::CStr::from_ptr(info.name) }.to_str().unwrap(), "demo");
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(info.version) }
                .to_str()
                .unwrap(),
            "1.2.3"
        );
    }

    fn ok_handler(hook: &str, payload: serde_json::Value) -> Result<serde_json::Value, i32> {
        Ok(serde_json::json!({"hook": hook, "payload": payload}))
    }

    fn err_handler(_hook: &str, _payload: serde_json::Value) -> Result<serde_json::Value, i32> {
        Err(7)
    }

    #[test]
    fn dispatch_invoke_handles_null_and_ok() {
        let result = unsafe { dispatch_invoke(std::ptr::null(), std::ptr::null(), ok_handler) };
        assert_eq!(result.status, 0);
        assert!(!result.json_output.is_null());
        let parsed: serde_json::Value = unsafe { parse_payload(result.json_output) }.unwrap();
        assert_eq!(parsed["hook"], "");
        assert_eq!(parsed["payload"], serde_json::json!({}));
        unsafe {
            let _ = CString::from_raw(result.json_output as *mut std::ffi::c_char);
        }
    }

    #[test]
    fn dispatch_invoke_parses_payload_and_errors() {
        let hook = CString::new("before_transform").unwrap();
        let payload = CString::new(r#"{"input":"x"}"#).unwrap();
        let ok = unsafe { dispatch_invoke(hook.as_ptr(), payload.as_ptr(), ok_handler) };
        assert_eq!(ok.status, 0);
        let parsed: serde_json::Value = unsafe { parse_payload(ok.json_output) }.unwrap();
        assert_eq!(parsed["hook"], "before_transform");
        assert_eq!(parsed["payload"]["input"], "x");
        unsafe {
            let _ = CString::from_raw(ok.json_output as *mut std::ffi::c_char);
        }

        let err = unsafe { dispatch_invoke(hook.as_ptr(), payload.as_ptr(), err_handler) };
        assert_eq!(err.status, 7);
        assert!(err.json_output.is_null());
    }
}
