//! Route elevations from a DEM, not from screenshot ascent totals.
//! Provider: https://www.opentopodata.org/api/ (WGS84, metres).
use super::model::Track;
use serde_json::Value;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const ENDPOINT: &str = "https://api.opentopodata.org/v1/srtm30m";
static LAST_CALL: Mutex<Option<Instant>> = Mutex::new(None);

pub struct ElevationInfo { pub samples: usize, pub ascent_m: f64 }

/// Internal generated points use BD09 even though the legacy field is gLat.
/// Invert both offsets before querying a WGS84 raster.
pub fn bd_to_wgs(lat: f64, lon: f64) -> (f64,f64) {
    let (glat,glon)=super::wire::bd09_to_gcj02(lat,lon);
    let (mut wlat,mut wlon)=(glat,glon);
    for _ in 0..8 {
        let (a,b)=super::geom::wgs84_to_gcj02(wlat,wlon);
        wlat-=a-glat; wlon-=b-glon;
    }
    (wlat,wlon)
}

/// Reject missing, reordered, out-of-range and malformed responses atomically.
pub fn parse_response(value: &Value, requested: &[(f64,f64)]) -> Result<Vec<f64>,String> {
    if value["status"] != "OK" { return Err("地形服务返回失败状态".into()); }
    let rows=value["results"].as_array().ok_or("地形响应缺少结果")?;
    if rows.len()!=requested.len() { return Err("地形采样数量不一致".into()); }
    rows.iter().zip(requested).map(|(r,&(lat,lon))| {
        let a=r["location"]["lat"].as_f64().ok_or("地形响应缺少纬度")?;
        let b=r["location"]["lng"].as_f64().ok_or("地形响应缺少经度")?;
        if (a-lat).abs()>0.00001 || (b-lon).abs()>0.00001 { return Err("地形返回坐标与请求不一致".into()); }
        let h=r["elevation"].as_f64().ok_or("路线有地形高程缺失；请更换路线或稍后重试")?;
        if !h.is_finite() || !(-500.0..=9000.0).contains(&h) { return Err("地形高程超出有效范围".into()); }
        Ok(h)
    }).collect()
}

/// Smooth on distance-spaced raster samples, then interpolate to the original
/// time grid. Do not add noise, force nonzero ascent, or match screenshot totals.
pub fn apply_samples(track: &mut Track, distances: &[f64], heights: &[f64]) -> Result<ElevationInfo,String> {
    if distances.len()<2 || distances.len()!=heights.len() || track.locations.is_empty()
        || heights.iter().any(|h| !h.is_finite())
        || distances.iter().any(|d| !d.is_finite())
        || distances.windows(2).any(|w| w[1]<=w[0])
        || distances[0].abs()>0.01
        || (distances[distances.len()-1]-track.totalDistance).abs()>0.01 {
        return Err("地形采样不完整或顺序无效".into());
    }
    let mut filtered=heights.to_vec();
    for i in 1..heights.len()-1 {
        filtered[i]=(heights[i-1]+2.0*heights[i]+heights[i+1])/4.0;
    }
    // A 1m deadband removes quantization chatter on integer-valued rasters.
    // Endpoint remains measured. All derived statistics use this same profile.
    let mut held=filtered[0];
    for i in 1..filtered.len()-1 {
        if (filtered[i]-held).abs()<1.0 { filtered[i]=held; } else { held=filtered[i]; }
    }
    let mut j=1;
    for point in &mut track.locations {
        while j<distances.len()-1 && distances[j]<point.totalDis { j+=1; }
        let k=((point.totalDis-distances[j-1])/(distances[j]-distances[j-1])).clamp(0.0,1.0);
        point.bdA=filtered[j-1]+(filtered[j]-filtered[j-1])*k;
        point.hasAltitude=true;
    }
    Ok(ElevationInfo{samples:distances.len(),ascent_m:super::altitude::total_ascent(&track.locations)})
}

fn sample_indices(track: &Track) -> Vec<usize> {
    let spacing=(track.totalDistance/98.0).max(30.0);
    let last=track.locations.len()-1;
    let endpoint=track.locations[last].totalDis;
    let mut indices=vec![0usize];
    let mut last_d=0.0;
    for (i,p) in track.locations.iter().enumerate().take(last).skip(1) {
        if p.totalDis-last_d>=spacing && p.totalDis<endpoint {
            indices.push(i);last_d=p.totalDis;
        }
    }
    indices.push(last);
    indices
}

pub fn apply_online(track: &mut Track) -> Result<ElevationInfo,String> {
    track.validate_consistency()?;
    let indices=sample_indices(track);
    if indices.len()>100 { return Err("地形采样超出服务限制".into()); }
    let coords: Vec<_>=indices.iter().map(|&i| {
        let p=&track.locations[i]; let (a,b)=bd_to_wgs(p.gLat,p.gLng);
        // Match the exact decimal coordinates sent in the request.
        ((a*1e6).round()/1e6,(b*1e6).round()/1e6)
    }).collect();
    let positions=coords.iter().map(|(a,b)|format!("{a:.6},{b:.6}")).collect::<Vec<_>>().join("|");
    let mut guard=LAST_CALL.lock().map_err(|_|"地形查询锁不可用")?;
    if let Some(last)=*guard {
        if last.elapsed()<Duration::from_millis(1100) { std::thread::sleep(Duration::from_millis(1100)-last.elapsed()); }
    }
    *guard=Some(Instant::now());
    // No account identifiers, password or token is sent to the elevation host.
    let agent=ureq::AgentBuilder::new().timeout(Duration::from_secs(25)).build();
    let response=agent.get(ENDPOINT).query("locations",&positions)
        .query("interpolation","bilinear").query("nodata_value","null").call()
        .map_err(|_|"地形地图加载失败：请检查网络或稍后重试；未使用随机海拔".to_string())?;
    let value: Value=response.into_json().map_err(|_|"地形服务返回了无效数据")?;
    let heights=parse_response(&value,&coords)?;
    let distances:Vec<_>=indices.iter().map(|&i|track.locations[i].totalDis).collect();
    apply_samples(track,&distances,&heights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn rounded_custom_endpoint_is_sampled_once() {
        let coords=[(39.9,116.4),(39.901,116.4),(39.901,116.401),(39.9,116.4)];
        let ring=super::super::custom::CustomRing{points_bd:coords.iter().map(|&(a,b)|super::super::geom::wgs84_to_bd09(a,b)).collect(),leg_length_m:336.646,total_length_m:336.646,loop_mode:true};
        let mut t=super::super::custom::build_custom(336.646,121,7,1700000000000,&ring,&[]);
        let ids=sample_indices(&t);
        let distances:Vec<_>=ids.iter().map(|&i|t.locations[i].totalDis).collect();
        assert_eq!(ids.iter().filter(|&&i|i==t.locations.len()-1).count(),1);
        assert!(distances.windows(2).all(|w|w[1]>w[0]));
        let out=apply_samples(&mut t,&distances,&vec![30.;distances.len()]).unwrap();
        assert_eq!(out.ascent_m,0.);
    }
    #[test]
    fn coordinate_roundtrip_stays_on_same_ground_cell() {
        for (a,b) in [(34.2,117.1),(39.9,116.4),(22.5,114.1)] {
            let (c,d)=super::super::geom::wgs84_to_bd09(a,b);
            let (e,f)=bd_to_wgs(c,d);
            assert!((a-e).abs()<0.000003 && (b-f).abs()<0.000003);
        }
    }
    #[test]
    fn missing_height_is_not_zero_and_reordered_response_is_rejected() {
        let coords=[(34.2,117.1)];
        let mut v=json!({"status":"OK","results":[{"location":{"lat":34.2,"lng":117.1},"elevation":null}]});
        assert!(parse_response(&v,&coords).is_err());
        v["results"][0]["elevation"]=json!(0.0);
        assert_eq!(parse_response(&v,&coords).unwrap(),vec![0.0]);
        v["results"][0]["location"]["lat"]=json!(35.2);
        assert!(parse_response(&v,&coords).is_err());
    }
    #[test]
    fn flat_ground_is_zero_and_slope_is_sample_rate_independent() {
        let mut track=super::super::generator::build(2000.0,720,42,(34.2,117.1),1700000000000,
            &[(34.2,117.1),(34.201,117.1),(34.201,117.101),(34.2,117.101)]);
        let distances=[0.0,500.0,1000.0,1500.0,2000.0];
        let out=apply_samples(&mut track,&distances,&[25.0;5]).unwrap();
        assert_eq!(out.ascent_m,0.0);
        let out=apply_samples(&mut track,&distances,&[25.0,30.0,35.0,40.0,45.0]).unwrap();
        assert_eq!(out.ascent_m,20.0);
        let mut sparse=track.locations.clone();sparse.retain(|p| p.totalTime%10==0);
        if sparse.last().unwrap().totalDis != 2000.0 { sparse.push(track.locations.last().unwrap().clone()); }
        assert!((super::super::altitude::total_ascent(&sparse)-20.0).abs()<0.01);
        assert!(track.validate_consistency().is_ok());
    }
}
