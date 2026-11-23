//! 生物力学联动：配速 v、步频 f_cadence、步幅 L_stride 的非线性耦合。
//!
//!   硬约束：v = (f_cadence / 60) · L_stride   （v: m/s，f: 步/分，L: 米）
//!
//! 由速度与疲劳度确定 (步频, 步幅)，**始终严格满足等式**。

/// 步态结果。
#[derive(Clone, Copy, Debug)]
pub struct Gait {
    /// 步频（步/分）。
    pub cadence: f64,
    /// 步幅（米）。
    pub stride: f64,
}

/// Screenshot summaries from two runners, 13 distinct runs, 2026-11.
/// (seconds/km, steps/min). No names or account IDs are embedded.
pub const REFERENCE_CADENCE: &[(f64, f64)] = &[
    (383.0,182.0),(358.0,184.0),(355.0,189.0),(336.0,183.0),(406.0,201.0),
    (237.0,173.0),(252.0,160.0),(287.0,161.0),(258.0,173.0),(265.0,160.0),
    (261.0,173.0),(260.0,158.0),(307.0,179.0),
];

/// Gaussian local mean, 20 seconds/km bandwidth. This is a descriptive
/// reference, not a physiological law or reconstruction of sensor readings.
/// Out-of-sample paces use the nearest supported boundary, never extrapolate.
pub fn reference_cadence(pace_s_km: f64) -> f64 {
    let pace = if pace_s_km.is_finite() { pace_s_km.clamp(237.0,406.0) } else { 358.0 };
    let mut weighted=0.0;
    let mut weights=0.0;
    for &(p,c) in REFERENCE_CADENCE {
        let w=(-0.5*((pace-p)/20.0).powi(2)).exp();
        weighted += w*c;
        weights += w;
    }
    weighted/weights
}

/// Speed determines reference cadence; stride is derived, not sampled
/// independently. Summary images do not establish a fatigue coefficient.
pub fn gait(v: f64, _fatigue: f64) -> Gait {
    if !v.is_finite() || v <= 0.0 { return Gait { cadence: 0.0, stride: 0.0 }; }
    let cadence=reference_cadence(1000.0/v);
    let stride=(v*60.0/cadence).clamp(0.5,1.7);
    Gait { cadence: v*60.0/stride, stride }
}

/// 由速度与疲劳度取步幅（米），包络 [0.55, 1.35]。
pub fn stride_for(v: f64, fatigue: f64) -> f64 {
    gait(v, fatigue).stride
}

/// 由速度反解步频（步/分）。
pub fn cadence_for(v: f64, stride: f64) -> f64 {
    (60.0 * v / stride.max(1e-6)).clamp(110.0, 220.0)
}

/// 由步频与步幅计算速度（m/s）。
pub fn speed_from(cadence: f64, stride: f64) -> f64 {
    (cadence / 60.0) * stride
}

/// 疲劳度（0..0.6）：由已跑里程（公里）线性抬升，0.5km 后开始，封顶 0.6。
pub fn fatigue_from_km(km: f64) -> f64 {
    if km <= 0.5 {
        0.0
    } else {
        ((km - 0.5) * 1.5).min(0.6)
    }
}

/// 步频 OU 耦合：在目标步频序列上做均值回归随机波动，返回逐点步频（步/分）。
///
///   c_{t+1} = c_t + θ(target_i − c_t)·Δt + σ·√Δt·Z,  Z~N(0,1)
///
/// 用于「步频-速度」耦合的随机部分，使步频既跟踪速度又不机械等值。
pub fn ou_cadence_series(
    targets: &[f64],
    dts: &[f64],
    seed: u64,
    theta: f64,
    sigma: f64,
    floor: f64,
    ceil: f64,
) -> Vec<f64> {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use rand_distr::{Distribution, Normal};

    let mut rng = StdRng::seed_from_u64(seed);
    let normal = Normal::<f64>::new(0.0, 1.0).unwrap();
    let mut out = Vec::with_capacity(targets.len());
    let mut c = targets.first().copied().unwrap_or(150.0);
    for i in 0..targets.len() {
        let dt = dts[i];
        c = c + theta * (targets[i] - c) * dt + sigma * dt.sqrt() * normal.sample(&mut rng);
        c = c.clamp(floor, ceil);
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_reference_has_two_cohorts_and_stays_bounded() {
        assert_eq!(REFERENCE_CADENCE.len(),13);
        for pace in 237..=406 {
            let g=gait(1000.0/pace as f64,0.0);
            assert!((158.0..=201.0).contains(&g.cadence));
            assert!((0.7..=1.6).contains(&g.stride));
            let next=reference_cadence(pace as f64+0.01);
            assert!((next-g.cadence).abs()<0.05);
        }
        assert!((reference_cadence(360.0)-185.0).abs()<5.0);
        assert!((reference_cadence(255.0)-167.0).abs()<5.0);
        assert_eq!(reference_cadence(100.0),reference_cadence(237.0));
        assert_eq!(reference_cadence(600.0),reference_cadence(406.0));
        assert_eq!(gait(0.0,0.0).cadence,0.0);
    }

    /// 等式约束 `v = cadence/60 * stride` 必须始终成立。
    #[test]
    fn cadence_and_stride_always_satisfy_the_speed_identity() {
        for v in [1.5, 2.0, 2.5, 3.0, 3.5, 4.0] {
            for fatigue in [0.0, 0.5, 1.0] {
                let g = gait(v, fatigue);
                let reconstructed = g.cadence / 60.0 * g.stride;
                assert!(
                    (reconstructed - v).abs() < 1e-9,
                    "v={v} fatigue={fatigue} 重建速度 {reconstructed} 不符"
                );
            }
        }
    }

    #[test]
    fn gait_coupling_equation_holds() {
        for v in [1.9f64, 2.5, 3.5, 5.0, 6.3] {
            for fat in [0.0f64, 0.3, 0.6] {
                let g = gait(v, fat);
                let back = speed_from(g.cadence, g.stride);
                assert!((back - v).abs() < 1e-6, "v={v} back={back}");
                assert!((0.5..=1.7).contains(&g.stride), "stride={}", g.stride);
                assert!((110.0..=230.0).contains(&g.cadence), "cad={}", g.cadence);
            }
        }
    }

    #[test]
    fn cadence_ou_stays_in_envelope() {
        let n = 200;
        let targets: Vec<f64> = (0..n).map(|i| 170.0 + 0.1 * i as f64).collect();
        let dts = vec![5.0; n];
        let cad = ou_cadence_series(&targets, &dts, 42, 0.25, 0.5, 110.0, 220.0);
        assert_eq!(cad.len(), n);
        for c in cad {
            assert!((110.0..=220.0).contains(&c), "cadence={c}");
        }
    }

    #[test]
    fn fatigue_monotonic_capped() {
        assert_eq!(fatigue_from_km(0.4), 0.0);
        assert!(fatigue_from_km(1.0) > 0.0);
        assert_eq!(fatigue_from_km(5.0), 0.6);
    }
}
