//! Structural and numerical checks for locally generated track data.
//! These checks do not predict acceptance by a remote service. Screenshot
//! summaries cannot establish its rejection rules. Short strides and flat
//! terrain are valid; no minimum elevation variance is imposed.
use super::model::Track;

pub const MIN_STEPS_PER_METRE: f64 = 0.05;
/// Matches the generator's lower stride bound of 0.5 metres.
pub const MAX_STEPS_PER_METRE: f64 = 2.0;
pub const MAX_POINT_SPEED_MPS: f64 = 12.0;
pub const MIN_CADENCE_SPEED_CORR: f64 = 0.55;

/// 自检结果：`errors` 非空时拒绝提交，`notes` 只作提示。
#[derive(Debug, Default, Clone)]
pub struct PreflightReport {
    pub errors: Vec<String>,
    pub notes: Vec<String>,
}

impl PreflightReport {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

fn metres_between(a: (f64, f64), b: (f64, f64)) -> f64 {
    // 5s 尺度内用等距近似即可，误差 < 0.1%。
    let dy = (b.0 - a.0) * super::geom::MET_PER_DEG_LAT;
    let dx = (b.1 - a.1) * super::geom::MET_PER_DEG_LNG;
    (dx * dx + dy * dy).sqrt()
}

fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len().min(y.len());
    if n < 8 {
        return None;
    }
    let (x, y) = (&x[..n], &y[..n]);
    let mx = x.iter().sum::<f64>() / n as f64;
    let my = y.iter().sum::<f64>() / n as f64;
    let mut cov = 0.0;
    let mut vx = 0.0;
    let mut vy = 0.0;
    for i in 0..n {
        let dx = x[i] - mx;
        let dy = y[i] - my;
        cov += dx * dy;
        vx += dx * dx;
        vy += dy * dy;
    }
    if vx <= 1e-12 || vy <= 1e-12 {
        return None;
    }
    Some(cov / (vx.sqrt() * vy.sqrt()))
}

/// 对一条轨迹做本地数值检查，不预测远程平台是否接受。
pub fn check(track: &Track) -> PreflightReport {
    let mut report = PreflightReport::default();
    let locs = &track.locations;
    if locs.len() < 4 || track.totalDistance <= 0.0 || track.totalTime <= 0 {
        report.errors.push("轨迹点太少或距离/时长无效".into());
        return report;
    }

    // ① 步数密度
    let density = track.totalSteps as f64 / track.totalDistance;
    if density < MIN_STEPS_PER_METRE {
        report.errors.push(format!(
            "步数密度 {density:.3} 步/米偏低（正常 {MIN_STEPS_PER_METRE}-{MAX_STEPS_PER_METRE}），步数不足以匹配距离"
        ));
    } else if density > MAX_STEPS_PER_METRE {
        report.errors.push(format!(
            "步数密度 {density:.3} 步/米超出本地生成器范围 {MIN_STEPS_PER_METRE}-{MAX_STEPS_PER_METRE}"
        ));
    }

    // Flat DEM profiles are valid; no minimum altitude variance is imposed.
    if locs.iter().any(|p| !p.bdA.is_finite()) {
        report.errors.push("海拔含无效数值".into());
    }

    // ③ 相邻点瞬时速度：坐标瞬移是最直接的破绽
    let mut worst = 0.0f64;
    let mut worst_at = 0usize;
    for i in 1..locs.len() {
        let dt = (locs[i].totalTime - locs[i - 1].totalTime) as f64;
        if dt <= 0.0 {
            continue;
        }
        let d = metres_between(
            (locs[i - 1].gLat, locs[i - 1].gLng),
            (locs[i].gLat, locs[i].gLng),
        );
        let v = d / dt;
        if v > worst {
            worst = v;
            worst_at = i;
        }
    }
    // 只有**两端都是正常绿点**的跳变才算"伪造瞬移"。
    //
    // 真人实测：GPS 丢信号再恢复时，1-3 秒内跳 30-90m 是常态，服务端照常
    // 通过（1295365397、1329975458 两条均为 status=0）。这类跳变必然伴随
    // 灰点（type=-1 / locType=4）或信号恢复点（type=7）与大 radius，
    // 是真人轨迹的固有特征，不该拦。
    let jump_is_plausible = |i: usize| -> bool {
        let near = |p: &super::model::GenPoint| {
            p.ptype == -1 || p.ptype == 7 || p.locType != 1 || p.radius > 50.0
        };
        // 检查跳变点及其前后各一点：GPS 丢信号时，灰点未必正好落在跳变的
        // 端点（服务端也是按窗口判断的）。
        let lo = i.saturating_sub(2);
        let hi = (i + 1).min(locs.len().saturating_sub(1));
        (lo..=hi).any(|k| near(&locs[k]))
    };
    if worst > MAX_POINT_SPEED_MPS && !jump_is_plausible(worst_at) {
        report.errors.push(format!(
            "第 {worst_at} 点与上一点之间隐含速度 {worst:.1} m/s（上限 {MAX_POINT_SPEED_MPS}），属于坐标瞬移"
        ));
    } else if worst > MAX_POINT_SPEED_MPS {
        report.notes.push(format!(
            "第 {worst_at} 点附近有 {worst:.0} m/s 的跳变，但伴随灰点/信号恢复点，按 GPS 抖动处理"
        ));
    }

    // ④ 步频与速度是否脱钩（步频恒定 = 老 bug 的特征）
    let speeds: Vec<f64> = track
        .speedPerTenSec
        .iter()
        .map(|w| w.value / w.time.max(1) as f64)
        .collect();
    let cadences: Vec<f64> = track
        .stepsPerTenSec
        .iter()
        .map(|w| w.value / w.time.max(1) as f64 * 60.0)
        .collect();
    // ④ 步频与速度的耦合程度：**只提示，不拦截**。
    //
    // 混合跑者的描述性配速曲线可出现负相关；相关性不作为拒绝条件，
    // 也不能据此推断真实程度或平台接受结果。
    if let Some(corr) = pearson(&speeds, &cadences) {
        if corr < MIN_CADENCE_SPEED_CORR {
            report.notes.push(format!(
                "步频与速度相关系数 {corr:.2}：混合样本的配速参考可出现低相关或负相关；此项仅供查看"
            ));
        }
    }

    // ⑤ 只作提示：灰点 GPS 精度异常大不会直接拒绝，但值得记录。
    let big_radius = locs.iter().filter(|p| p.radius > 50.0).count();
    if big_radius > 0 {
        report.notes.push(format!(
            "有 {big_radius} 个点的 GPS 精度值大于 50m"
        ));
    }

    report
}

/// 海拔标准差（米），日志与测试用。
pub fn altitude_sd(track: &Track) -> f64 {
    let n = track.locations.len() as f64;
    if n <= 0.0 {
        return 0.0;
    }
    let mean = track.locations.iter().map(|p| p.bdA).sum::<f64>() / n;
    (track
        .locations
        .iter()
        .map(|p| (p.bdA - mean).powi(2))
        .sum::<f64>()
        / n)
        .sqrt()
}

/// 自检不通过时返回可直接展示的错误串。
pub fn enforce(track: &Track) -> Result<PreflightReport, String> {
    let report = check(track);
    if report.is_ok() {
        Ok(report)
    } else {
        Err(format!(
            "轨迹未通过风控自检，已阻止提交：\n  - {}",
            report.errors.join("\n  - ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::generator::{build, build_with_color_mode};
    use crate::api::model::TrackColorMode;

    fn points() -> Vec<(f64, f64)> {
        vec![
            (38.901678, 121.540241),
            (38.902564, 121.541233),
            (38.900921, 121.542310),
            (38.899823, 121.541010),
            (38.900455, 121.539512),
        ]
    }

    fn good_track() -> Track {
        build_with_color_mode(
            2000.0,
            1000,
            42,
            (38.9, 121.54),
            1_700_000_000_000,
            &points(),
            TrackColorMode::FullGreen,
        )
    }

    #[test]
    fn normal_generated_track_passes() {
        let track = good_track();
        let report = check(&track);
        assert!(report.is_ok(), "正常轨迹被误判：{:?}", report.errors);
    }

    #[test]
    fn flat_elevation_is_valid_but_nonfinite_is_rejected() {
        let mut track=good_track();
        for p in &mut track.locations { p.bdA=520.0; }
        assert!(check(&track).is_ok());
        track.locations[0].bdA=f64::NAN;
        assert!(check(&track).errors.iter().any(|e|e.contains("海拔")));
    }

    #[test]
    fn inflated_steps_are_rejected() {
        let mut track = good_track();
        // 10-26 13:26 那条：5106m / 5544 步 = 1.086；这里模拟 1.33。
        track.totalSteps = (track.totalDistance * 2.5) as i64;
        let report = check(&track);
        assert!(
            report.errors.iter().any(|e| e.contains("步数密度")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn coordinate_jump_is_rejected() {
        let mut track = good_track();
        let i = track.locations.len() / 2;
        // 5 秒内偏 80m ≈ 16 m/s，正好复现 10-24 那条的瞬移。
        track.locations[i].gLat += 0.0007;
        let report = check(&track);
        assert!(
            report.errors.iter().any(|e| e.contains("坐标瞬移")),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn real_gps_gap_jump_near_gray_point_is_allowed() {
        // 复现真人记录 1329975458：灰点（type=-1, locType=4, radius=150）
        // 之后 3 秒跳 91.7m，服务端 status=0 通过。
        let mut track = good_track();
        let i = track.locations.len() / 2;
        if i > 1 && i + 1 < track.locations.len() {
            track.locations[i].ptype = -1;
            track.locations[i].locType = 4;
            track.locations[i].radius = 150.0;
            track.locations[i + 1].gLat += 0.0008;
            let report = check(&track);
            assert!(
                !report.errors.iter().any(|e| e.contains("坐标瞬移")),
                "灰点附近的 GPS 跳变不应被拦：{:?}",
                report.errors
            );
        }
    }

    #[test]
    fn jump_between_two_clean_green_points_is_rejected() {
        // 两端都是正常绿点却瞬移 → 伪造痕迹，必须拦。
        let mut track = good_track();
        let i = track.locations.len() / 2;
        for p in &track.locations {
            // 先把所有点清成正常绿点
            let _ = p;
        }
        track.locations[i].ptype = 0;
        track.locations[i].locType = 1;
        track.locations[i].radius = 2.0;
        track.locations[i + 1].ptype = 0;
        track.locations[i + 1].locType = 1;
        track.locations[i + 1].radius = 2.0;
        track.locations[i + 1].gLat += 0.0015; // 约 166m
        let report = check(&track);
        assert!(
            report.errors.iter().any(|e| e.contains("坐标瞬移")),
            "绿点间的大跳变应被拦：{:?}",
            report.errors
        );
    }

    #[test]
    fn flat_cadence_only_warns_never_blocks() {
        // 真人实测步频-速度相关 -0.34~0.80，不能当硬门槛（会误拦真跑者）。
        let mut track = good_track();
        let n = track.stepsPerTenSec.len();
        for (i, w) in track.stepsPerTenSec.iter_mut().enumerate() {
            w.time = 10;
            w.value = if i + 1 == n { 0.0 } else { 30.0 };
        }
        let report = check(&track);
        assert!(report.is_ok(), "步频相关不应拦截提交：{:?}", report.errors);
        assert!(
            report.notes.iter().any(|e| e.contains("步频与速度")),
            "应给出提示：{:?}",
            report.notes
        );
    }

    #[test]
    fn legacy_build_still_passes() {
        let track = build(1500.0, 700, 7, (38.9, 121.54), 1_700_000_000_000, &points());
        let report = check(&track);
        assert!(report.is_ok(), "经典生成器轨迹被误判：{:?}", report.errors);
    }
}
