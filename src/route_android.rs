use std::{path::PathBuf,sync::{Mutex,OnceLock}};
use jni::{JNIEnv,objects::{JClass,JString},sys::jstring};
static DIRECTORY:OnceLock<PathBuf>=OnceLock::new();static LOCK:Mutex<()>=Mutex::new(());
pub fn data_dir()->PathBuf{DIRECTORY.get().expect("Android storage not initialized").clone()}
pub fn edit_text(_:u64,_:&str,_:crate::platform::InputKind){}pub fn take_edited_text(_:u64)->Option<String>{None}pub fn set_keep_screen_on(_:bool){}pub fn version_name()->String{"1.1.0".into()}pub fn install_apk(_:&str){}
#[no_mangle]pub extern "system" fn Java_org_sportssimulation_virtualrun_CoreBridge_call(mut env:JNIEnv,_class:JClass,path:JString,op:JString,input:JString)->jstring{
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->Result<serde_json::Value,String>{
  let _guard=LOCK.lock().map_err(|_|"核心状态异常，请重启应用".to_string())?;
  let dir:String=env.get_string(&path).map_err(|e|e.to_string())?.into();let operation:String=env.get_string(&op).map_err(|e|e.to_string())?.into();let raw:String=env.get_string(&input).map_err(|e|e.to_string())?.into();
  std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let _=DIRECTORY.set(PathBuf::from(dir));crate::route_app::dispatch(&operation,&raw)
 }));
 let value=match result{Ok(Ok(data))=>serde_json::json!({"ok":true,"data":data}),Ok(Err(e))=>serde_json::json!({"ok":false,"error":e}),Err(_)=>serde_json::json!({"ok":false,"error":"核心处理异常，请重新打开应用"})};
 match env.new_string(value.to_string()){Ok(s)=>s.into_raw(),Err(_)=>std::ptr::null_mut()}
}
