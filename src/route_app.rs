//! Account and selected-route adapter. Uses the existing protocol implementation.
use crate::api::{client::ApiClient, model, flow::{RunParams,run_full_flow_controlled}};
use crate::track::{custom::{CustomRing,CustomRouteSpec},geom,generate_road::RouteMode};
use serde_json::{json,Value};
#[derive(serde::Deserialize)]
pub struct Selection {pub points:Vec<(f64,f64)>,pub speed:f64,pub laps:i64,#[serde(rename="loop")]pub closed:bool,#[serde(default)]pub city:String,#[serde(default)]pub sport:String,#[serde(default)]pub seed:Option<u64>}
fn metres(a:(f64,f64),b:(f64,f64))->f64 {let x=(b.0-a.0).to_radians();let y=(b.1-a.1).to_radians();let h=(x/2.).sin().powi(2)+a.0.to_radians().cos()*b.0.to_radians().cos()*(y/2.).sin().powi(2);12742017.6*h.min(1.).sqrt().asin()}
pub fn prepare(s:&Selection)->Result<(CustomRing,RunParams),String>{
 if s.points.len()<2||s.points.len()>10000{return Err("需要 2～10000 个路线点".into());}
 if !s.speed.is_finite()||!(3.6..=20.).contains(&s.speed)||!(1..=100).contains(&s.laps){return Err("提交速度须为 3.6～20 km/h，圈数 1～100".into());}
 if s.points.iter().any(|p|!p.0.is_finite()||!p.1.is_finite()||p.0.abs()>85.||p.1.abs()>180.){return Err("WGS84 坐标无效".into());}
 let mut input=Vec::new();for &p in &s.points{if input.last().map(|&a|metres(a,p)>0.02).unwrap_or(true){input.push(p);}}
 if input.len()<2{return Err("路线点不能全部重合".into());}
 if s.closed&&metres(input[0],*input.last().unwrap())>0.02{input.push(input[0]);}
 let mut length=0.;for w in input.windows(2){let d=metres(w[0],w[1]);if d>10000.{return Err("相邻途经点超过 10 公里，请补充途经点".into());}length+=d;}
 let laps=if s.closed{s.laps}else{1};let dist=length*laps as f64;let dur=(dist/(s.speed/3.6)).round() as i64;
 if !(100.0..=20000.0).contains(&dist)||dur<60||dur>14400{return Err("提交路线总长须为 100 米～20 公里，用时须为 1 分钟～4 小时".into());}
 let ring=CustomRing{points_bd:input.iter().map(|p|geom::wgs84_to_bd09(p.0,p.1)).collect(),leg_length_m:length,total_length_m:length,loop_mode:s.closed};
 let spec=CustomRouteSpec{origin:"map-selected-start".into(),destination:"map-selected-end".into(),loop_mode:s.closed,laps};
 let params=RunParams{selected_route:Some(ring.clone()),dist,dur,start_ms:crate::crypto::envelope::now_ms()-dur*1000-5000,face_check:0,manual_altitude:None,manual_altitude_range:None,terrain_elevation:true,track_color_mode:model::TrackColorMode::FullGreen,seed:s.seed.unwrap_or_else(rand::random),route_mode:RouteMode::Custom,ordinary_pick:3,track_spec:crate::track::stadium::TrackSpec::Auto,custom:Some(spec),amap_key:String::new(),sport:crate::api::sport::SportKind::from_key(&s.sport)};
 Ok((ring,params))
}
fn preview_summary(t:&crate::track::model::Track,ascent:f64)->Value{
 let gait=route_planner::gait(t.totalDistance/t.totalTime as f64,0.);
 let mut profile=Vec::<[f64;2]>::new();for p in &t.locations{if profile.last().map(|q|p.totalDis>q[0]).unwrap_or(true){profile.push([p.totalDis,p.bdA]);}}
 json!({"message":"轨迹与地形校验通过，尚未提交","distance_m":t.totalDistance,"duration_s":t.totalTime,"points":t.locations.len(),"steps":t.totalSteps,"cadence_spm":t.totalSteps as f64*60./t.totalTime as f64,"stride_cm":t.totalDistance*100./t.totalSteps.max(1) as f64,"reference_cadence_spm":gait.cadence,"ascent_m":ascent,"terrain_source":"Open Topo Data / SRTM30m","elevation_profile":profile})
}
// Read-only account map adapter; no run creation or submission.
fn bd_to_wgs(p:(f64,f64))->(f64,f64){let(mut a,mut o)=p;for _ in 0..12{let q=geom::wgs84_to_bd09(a,o);a-=q.0-p.0;o-=q.1-p.1;}(a,o)}
fn account_map(v:&Value)->Result<Value,String>{
 let session=model::load_session();if !session.is_logged_in(){return Err("请先登录运动世界".into());}
 let mut client=ApiClient::new(model::load_identity(),Some(session));let mut warnings=Vec::<String>::new();
 let fences=match crate::api::fence::fetch_geo_fence_with_area(&mut client){Ok(f)=>f.fences,Err(e)=>{if crate::api::client::is_session_expired(&e){model::clear_session();return Err(e);}warnings.push(format!("校区边界：{e}"));Vec::new()}};
 let index=v["campus"].as_u64().unwrap_or(0) as usize;let index=if index<fences.len(){index}else{0};
 let (anchor,source,area)=if let Some(f)=fences.get(index){let n=f.ring.len() as f64;let p=(f.ring.iter().map(|p|p.0).sum::<f64>()/n,f.ring.iter().map(|p|p.1).sum::<f64>()/n);(Some(p),"所选校区中心",f.id.map(|x|x.to_string()))}else if let(Some(a),Some(o))=(v["latitude"].as_f64(),v["longitude"].as_f64()){crate::location::Coordinate::new(a,o,0.)?;(Some(geom::wgs84_to_bd09(a,o)),"手机位置",None)}else{(None,"未获得位置",None)};
 let must_points=match crate::api::policy::fetch_policy_for(&mut client,1){Ok(p)=>p.must_points,Err(e)=>{warnings.push(format!("必经点策略：{e}"));Vec::new()}};
 let mut checkpoints=Vec::<Value>::new();let mut from_cache=false;
 if let Some((a,o))=anchor{let coordinate=crate::location::Coordinate::new(a,o,0.)?;
  let mut log=|line:&str|{if line.contains("缓存"){from_cache=true;}};
  match crate::api::points::fetch_points_context_ext(&mut client,coordinate,area,&mut log){Ok(ctx)=>{for (i,p) in ctx.points.iter().enumerate(){if let Some(bd)=crate::api::fence::point_xy(p){let w=bd_to_wgs(bd);let name=p.get("name").or_else(||p.get("pointName")).and_then(Value::as_str).map(str::to_string).unwrap_or_else(||format!("点位 {}",i+1));let fixed=p.get("isFixed").map(|x|x.as_i64()==Some(1)||x.as_str()==Some("1")||x.as_bool()==Some(true)).unwrap_or(false)||must_points.iter().any(|&q|metres(bd,q)<3.);checkpoints.push(json!({"name":name,"lat":w.0,"lon":w.1,"required":fixed}));}}},Err(e)=>{if crate::api::client::is_session_expired(&e){model::clear_session();return Err(e);}warnings.push(format!("路线点位：{e}"));}}
 }else{warnings.push("未取得校区边界；请允许定位，定位成功后刷新。".into());}
 for (i,&bd) in must_points.iter().enumerate(){let w=bd_to_wgs(bd);if !checkpoints.iter().any(|p|metres((p["lat"].as_f64().unwrap_or(0.),p["lon"].as_f64().unwrap_or(0.)),w)<3.){checkpoints.push(json!({"name":format!("必经点 {}",i+1),"lat":w.0,"lon":w.1,"required":true}));}}
 let campus:Vec<Value>=fences.iter().enumerate().map(|(i,f)|json!({"index":i,"name":if f.name.is_empty(){format!("校区 {}",i+1)}else{f.name.clone()},"ring":f.ring.iter().map(|&p|bd_to_wgs(p)).collect::<Vec<_>>()})).collect();
 Ok(json!({"checkpoints":checkpoints,"campuses":campus,"selected_campus":index,"source":if from_cache{"运动世界接口缓存"}else{"运动世界接口"},"anchor_source":source,"warnings":warnings,"street_map_available":false,"fetched_at_ms":crate::crypto::envelope::now_ms()}))
}
pub fn dispatch(op:&str,raw:&str)->Result<Value,String>{
 if raw.len()>2_000_000{return Err("输入过大".into());}
 let v:Value=serde_json::from_str(raw).map_err(|e|e.to_string())?;
 match op{
 "mapData"=>account_map(&v),
 "session"=>{let s=model::load_session();Ok(json!({"logged_in":s.is_logged_in(),"name":s.name}))},
 "login"=>{let u=v["username"].as_str().unwrap_or("").trim();let p=v["password"].as_str().unwrap_or("");if u.is_empty()||p.is_empty(){return Err("请填写运动世界账号和密码".into());}let mut client=ApiClient::new(model::load_identity(),None);let s=crate::api::login::login_for_route_app(&mut client,u,p,v.get("captcha"),&mut |_|{})?;model::save_session(&s)?;let _=std::fs::remove_file(crate::platform::data_dir().join("points_cache.json"));Ok(json!({"logged_in":true,"name":s.name,"message":"运动世界登录成功"}))},
 "logout"=>{let session=model::load_session();if !session.is_logged_in(){return Ok(json!({"logged_in":false,"server_logout":false,"message":"本机没有可用登录会话，无法退出其他设备。若提示 10121，请在原设备退出或按提示等待一小时。"}));}let mut client=ApiClient::new(model::load_identity(),Some(session));client.call("POST",crate::api::login::LOGOUT_PATH,"{}",&[]).map_err(|e|format!("服务端退出未成功，已保留本机会话供重试：{e}"))?;model::clear_session();let _=std::fs::remove_file(crate::platform::data_dir().join("points_cache.json"));Ok(json!({"logged_in":false,"server_logout":true,"message":"服务端已确认退出，本机登录会话已清理"}))},
 "preview"|"submit"=>{
  let s:Selection=serde_json::from_value(v).map_err(|e|e.to_string())?;let(ring,params)=prepare(&s)?;
  if op=="preview"{let mut t=crate::track::custom::build_custom(params.dist,params.dur,params.seed,params.start_ms,&ring,&[]);t.validate_consistency()?;let terrain=crate::track::elevation::apply_online(&mut t)?;return Ok(preview_summary(&t,terrain.ascent_m));}
  if s.city.trim().is_empty(){return Err("请填写本次路线所在城市".into());}if !["campus","outdoor"].contains(&s.sport.as_str()){return Err("请选择校园跑或户外跑".into());}
  let session=model::load_session();if !session.is_logged_in(){return Err("请先登录运动世界账号".into());}
  let mut identity=model::load_identity();let anchor=geom::wgs84_to_bd09(s.points[0].0,s.points[0].1);identity.anchor_lat=anchor.0;identity.anchor_lon=anchor.1;identity.city=s.city.trim().into();model::save_identity(&identity)?;
  let mut client=ApiClient::new(identity,Some(session));let mut logs=Vec::<String>::new();
  let result=run_full_flow_controlled(&mut client,&params,&mut |line|{if logs.len()<300{logs.push(if line.contains("token")||line.contains("password")||line.contains("session"){"[认证日志已省略]".into()}else{line.to_string()});}},&crate::task_control::Control::default());
  match result{Ok(o)=>Ok(json!({"message":"服务端已返回提交结果","record_id":o.result.rrid,"distance_m":o.result.total_dis,"duration_s":o.result.total_time,"obs_ok":o.obs_ok,"detail_ok":o.detail_ok,"complete":o.result.complete,"logs":logs})),Err(e)=>{if crate::api::client::is_session_expired(&e){model::clear_session();}Err(e)}}
 },_=>Err("未知操作".into())}
}
#[cfg(test)]mod tests{use super::*;
 fn sample()->Selection{Selection{points:vec![(39.9,116.4),(39.901,116.4),(39.901,116.401)],speed:8.,laps:2,closed:true,city:"北京市".into(),sport:"outdoor".into(),seed:Some(7)}}
 #[test]fn selected_route_preserves_order_and_closes(){let s=sample();let(r,p)=prepare(&s).unwrap();assert_eq!(r.points_bd.len(),4);assert_eq!(r.points_bd[0],geom::wgs84_to_bd09(39.9,116.4));assert_eq!(r.points_bd[0],r.points_bd[3]);assert!((p.dist-2.*r.total_length_m).abs()<0.001);assert!(p.selected_route.is_some());assert!(p.terrain_elevation);}
 #[test]fn open_route_does_not_add_return_leg(){let mut s=sample();s.closed=false;let(r,p)=prepare(&s).unwrap();assert_eq!(r.points_bd.len(),3);assert!((p.dist-r.total_length_m).abs()<0.001);}
 #[test]fn invalid_route_is_rejected(){let mut s=sample();s.speed=0.;assert!(prepare(&s).is_err());s.speed=8.;s.points[1].0=f64::NAN;assert!(prepare(&s).is_err());}
 #[test]fn preview_uses_original_track_builder_offline(){let s=sample();let(r,p)=prepare(&s).unwrap();let t=crate::track::custom::build_custom(p.dist,p.dur,7,p.start_ms,&r,&[]);t.validate_consistency().unwrap();assert!((t.totalDistance-p.dist).abs()<1.);assert!(!t.locations.is_empty());}
 #[test]fn preview_summary_uses_one_terrain_curve_and_consistent_metrics(){let s=sample();let(r,p)=prepare(&s).unwrap();let mut t=crate::track::custom::build_custom(p.dist,p.dur,7,p.start_ms,&r,&[]);let dist=t.totalDistance;let terrain=crate::track::elevation::apply_samples(&mut t,&[0.,dist/2.,dist],&[20.,30.,40.]).unwrap();t.locations.insert(1,t.locations[0].clone());let v=preview_summary(&t,terrain.ascent_m);assert_eq!(v["ascent_m"],20.);let profile=v["elevation_profile"].as_array().unwrap();assert!(profile.windows(2).all(|w|w[1][0].as_f64().unwrap()>w[0][0].as_f64().unwrap()));assert!((profile.last().unwrap()[0].as_f64().unwrap()-dist).abs()<0.01);let cadence=v["cadence_spm"].as_f64().unwrap();let stride=v["stride_cm"].as_f64().unwrap()/100.;assert!((cadence/60.*stride-dist/t.totalTime as f64).abs()<1e-8);}
 #[test]#[ignore="Public DEM request; no account or submission"]fn live_terrain_preview_without_account(){let v=dispatch("preview",r#"{"points":[[39.9,116.4],[39.901,116.4],[39.901,116.401]],"speed":10,"laps":1,"loop":true,"seed":7}"#).unwrap();assert!(v["ascent_m"].as_f64().unwrap()>=0.);assert!(v["elevation_profile"].as_array().unwrap().len()>2);println!("DEM_PREVIEW_OK points={} ascent_m={} cadence_spm={}",v["points"],v["ascent_m"],v["cadence_spm"]);}

}
