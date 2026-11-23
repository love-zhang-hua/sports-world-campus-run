//! Narrow C ABI for the iOS host. Owned return strings must be freed by route_free.
use std::{ffi::{CStr, CString, c_char}, path::PathBuf, sync::{Mutex, OnceLock}};
use serde_json::{json, Value};
static DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
static CALL_LOCK: Mutex<()> = Mutex::new(());
pub fn data_dir() -> PathBuf { DIRECTORY.get().expect("iOS storage not initialized").clone() }
unsafe fn text<'a>(p: *const c_char) -> Result<&'a str, String> {
    if p.is_null() { return Err("空参数".into()); }
    CStr::from_ptr(p).to_str().map_err(|_| "参数编码无效".into())
}
#[no_mangle]
pub unsafe extern "C" fn route_call(directory: *const c_char, operation: *const c_char, input: *const c_char) -> *mut c_char {
    let result = std::panic::catch_unwind(|| -> Result<Value, String> {
        let _guard = CALL_LOCK.lock().map_err(|_| "核心忙，请重启应用")?;
        let dir = PathBuf::from(text(directory)?);
        if !dir.is_absolute() { return Err("需要应用私有目录".into()); }
        std::fs::create_dir_all(&dir).map_err(|_| "无法创建应用存储目录")?;
        let stored = DIRECTORY.get_or_init(|| dir.clone());
        if *stored != dir { return Err("不允许切换会话目录".into()); }
        let op = text(operation)?;
        if !["session", "login", "logout", "mapData", "preview", "submit"].contains(&op) {
            return Err("未知操作".into());
        }
        crate::route_app::dispatch(op, text(input)?)
    });
    let value = match result {
        Ok(Ok(data)) => json!({"ok":true,"data":data}),
        Ok(Err(error)) => json!({"ok":false,"error":error}),
        Err(_) => json!({"ok":false,"error":"核心处理异常，请重新打开应用"})
    };
    CString::new(value.to_string()).map(CString::into_raw).unwrap_or(std::ptr::null_mut())
}
#[no_mangle]
pub unsafe extern "C" fn route_free(value: *mut c_char) {
    if !value.is_null() { drop(CString::from_raw(value)); }
}
