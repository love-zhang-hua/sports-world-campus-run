//! Running reference model, with cadence in steps/min and length in metres/step.
//! Identity: speed = cadence / 60 * step_length.
//! This is a population reference, not an individual optimum or sensor reading.

#[derive(Clone, Copy, Debug)]
pub struct Gait {
    pub cadence: f64,
    /// Historical API name; this is ONE step, not a two-step stride cycle.
    pub stride: f64,
}

/// Screenshot summaries from two runners, 13 distinct runs, 2026-11.
/// (seconds/km, steps/min). No names or account IDs are embedded.
pub const REFERENCE_CADENCE: &[(f64, f64)] = &[
    (383.0,182.0),(358.0,184.0),(355.0,189.0),(336.0,183.0),(406.0,201.0),
    (237.0,173.0),(252.0,160.0),(287.0,161.0),(258.0,173.0),(265.0,160.0),
    (261.0,173.0),(260.0,158.0),(307.0,179.0),
];

/// Van Oeveren et al. (2019), DOI 10.1080/02640414.2019.1614137.
/// SF in strides/min = 75.01 + 3.006 * speed_m_s. Multiply by TWO for steps/min.
/// The source population speed range is 1.64..=4.68 m/s; outside that range
/// hold the boundary cadence as an engineering fallback, not validated running.
/// The two screenshot cohorts remain observations, not a pooled speed model.
pub const REFERENCE_SPEED_MIN: f64 = 1.64;
pub const REFERENCE_SPEED_MAX: f64 = 4.68;

pub fn reference_speed_supported(v: f64) -> bool {
    v.is_finite() && (REFERENCE_SPEED_MIN..=REFERENCE_SPEED_MAX).contains(&v)
}

pub fn reference_cadence(pace_s_km: f64) -> f64 {
    if !pace_s_km.is_finite() || pace_s_km <= 0.0 { return 0.0; }
    let v=(1000.0/pace_s_km).clamp(REFERENCE_SPEED_MIN,REFERENCE_SPEED_MAX);
    150.02 + 6.012*v
}

/// No fatigue/grade coefficient can be identified from summary screenshots.
/// Derive step length at the ACTUAL speed so the identity also holds outside
/// the evidence range. Do not clamp length then silently alter cadence.
pub fn gait(v: f64, _fatigue: f64) -> Gait {
    if !v.is_finite() || v <= 0.0 { return Gait { cadence: 0.0, stride: 0.0 }; }
    let cadence=reference_cadence(1000.0/v);
    Gait { cadence, stride: v*60.0/cadence }
}

/// Step length (metres/step), derived from actual speed and reference cadence.
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
    fn population_reference_converts_strides_to_steps_and_preserves_observations() {
        assert_eq!(REFERENCE_CADENCE.len(),13);
        // Published coefficients, including the often-missed factor of two.
        assert!((reference_cadence(360.0)-166.72).abs()<1e-9);
        assert!((reference_cadence(240.0)-175.07).abs()<1e-9);
        let mut last=0.0;
        for n in 164..=468 {
            let v=n as f64/100.0;
            let g=gait(v,0.0);
            assert!(g.cadence>last);
            assert!((g.cadence-(150.02+6.012*v)).abs()<1e-9);
            assert!((speed_from(g.cadence,g.stride)-v).abs()<1e-9);
            last=g.cadence;
        }
    }

    #[test]
    fn unsupported_speeds_hold_boundary_cadence_without_breaking_identity() {
        assert!(reference_speed_supported(1.64));
        assert!(reference_speed_supported(4.68));
        for v in [0.1,1.0,1.63,4.69,6.3,10.0] {
            assert!(!reference_speed_supported(v));
            let g=gait(v,0.0);
            let boundary=if v<1.64 {1.64} else {4.68};
            assert!((g.cadence-gait(boundary,0.0).cadence).abs()<1e-9);
            assert!((speed_from(g.cadence,g.stride)-v).abs()<1e-9);
        }
        for v in [0.0,-1.0,f64::NAN,f64::INFINITY] {
            assert!(!reference_speed_supported(v));
            assert_eq!(gait(v,0.0).cadence,0.0);
        }
        for pace in [0.0,-1.0,f64::NAN,f64::INFINITY] {
            assert_eq!(reference_cadence(pace),0.0);
        }
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
                assert!(g.stride.is_finite() && g.stride > 0.0, "stride={}", g.stride);
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
