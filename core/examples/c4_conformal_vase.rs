//! Exact analytic Claude vase chart diagnostic, NOT production C2/C3 acceptance.
use std::f64::consts::{PI, TAU};
use vstone_core::{
    model::CoreInput,
    planes::{self, SurfaceQueries},
    seeds::Seed,
    topology::Topology,
};
struct Vase {
    y: Vec<f64>,
    v: Vec<f64>,
}
fn r(y: f64) -> f64 {
    if y < 0. {
        6. + PI / 10. * y
    } else if y > 40. {
        6. - PI / 10. * (y - 40.)
    } else {
        6. + 4. * (PI * y / 40.).sin()
    }
}
fn dr(y: f64) -> f64 {
    PI / 10. * (PI * y.clamp(0., 40.) / 40.).cos()
}
fn interp(x: f64, xs: &[f64], ys: &[f64]) -> f64 {
    if x < xs[0] {
        return ys[0];
    }
    if x > xs[xs.len() - 1] {
        return ys[ys.len() - 1];
    }
    let j = xs.partition_point(|v| *v < x).clamp(1, xs.len() - 1);
    let f = ((x - xs[j - 1]) / (xs[j] - xs[j - 1])).clamp(0., 1.);
    ys[j - 1] + f * (ys[j] - ys[j - 1])
}
impl Vase {
    fn new() -> Self {
        let y: Vec<_> = (0..=20000)
            .map(|i| -25. + 90. * i as f64 / 20000.)
            .collect();
        let dv: Vec<_> = y
            .iter()
            .map(|&y| (1. + dr(y).powi(2)).sqrt() / r(y))
            .collect();
        let mut v = vec![0.];
        for i in 1..y.len() {
            v.push(v[i - 1] + (dv[i] + dv[i - 1]) / 2. * (y[i] - y[i - 1]));
        }
        let zero = interp(0., &y, &v);
        for x in &mut v {
            *x -= zero;
        }
        Self { y, v }
    }
    fn y(&self, v: f64) -> f64 {
        interp(v, &self.v, &self.y)
    }
    fn topology(&self, n: usize, d: f64) -> Topology {
        let du = TAU / n as f64;
        let step = du * 3f64.sqrt() / 2.;
        let top = interp(40., &self.y, &self.v);
        let mut seeds = vec![];
        for k in -3i32..=(top / step).ceil() as i32 + 3 {
            for j in 0..n {
                let v = k as f64 * step;
                let y = self.y(v);
                seeds.push(Seed {
                    u: j as f64 * du + if k.rem_euclid(2) == 1 { du / 2. } else { 0. },
                    v,
                    s: y,
                    row: (k + 3) as usize,
                    auxiliary: !(0. ..=40.).contains(&y),
                    depth: k.rem_euclid(3) as f64 * d / 2.,
                });
            }
        }
        Topology::build(seeds).unwrap()
    }
}
impl SurfaceQueries for Vase {
    fn frame(&self, uv: [f64; 2]) -> ([f64; 3], [f64; 3]) {
        let y = self.y(uv[1]);
        let a = uv[0];
        let q = (1. + dr(y).powi(2)).sqrt();
        (
            [r(y) * a.cos(), y, r(y) * a.sin()],
            [a.cos() / q, -dr(y) / q, a.sin() / q],
        )
    }
    fn inside_rims(&self, p: [f64; 3]) -> bool {
        (0. ..=40.).contains(&p[1])
    }
    // Closest point on revolved analytic profile, including tangent extensions.
    fn distance(&self, p: [f64; 3]) -> f64 {
        let rho = p[0].hypot(p[2]);
        let mut lo = -25.;
        let mut hi = 65.;
        for _ in 0..90 {
            let a = lo + (hi - lo) / 3.;
            let b = hi - (hi - lo) / 3.;
            let f = |y: f64| (rho - r(y)).powi(2) + (p[1] - y).powi(2);
            if f(a) < f(b) {
                hi = b;
            } else {
                lo = a;
            }
        }
        let y = (lo + hi) / 2.;
        ((rho - r(y)).powi(2) + (p[1] - y).powi(2)).sqrt() * (rho - r(p[1])).signum()
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let n = args[1].parse().unwrap();
    let d = args[2].parse().unwrap();
    let vase = Vase::new();
    let t = vase.topology(n, d);
    if args.iter().any(|s| s == "--chart-only") {
        println!(
            "{}",
            serde_json::json!({"topology":t,"points":t.tiles.iter().map(|s|vase.offset([s.u,s.v],s.depth)).collect::<Vec<_>>() })
        );
        return;
    }
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    input.params.depth = d;
    let result = planes::experimental_conformal_penalty(
        &vase,
        &t,
        &input.params,
        args.iter().any(|s| s == "--rim-guard").then_some([0., 40.]),
    )
    .unwrap();
    println!(
        "{}",
        serde_json::json!({"schema":"c4.conformal-vase/1","experimental_C2_C3_bypass":true,"columns":n,"depth":d,"params":input.params,"topology":t,"diagnostic":result})
    );
}
#[test]
fn lattice_pattern() {
    let v = Vase::new();
    for n in [8, 10] {
        let t = v.topology(n, 2.5);
        assert_eq!(t.tiles.len(), if n == 8 { 120 } else { 170 });
        for (i, s) in t.tiles.iter().enumerate() {
            let k = s.row as i32 - 3;
            assert_eq!(s.depth, k.rem_euclid(3) as f64 * 1.25);
            assert!((s.v - k as f64 * TAU / n as f64 * 3f64.sqrt() / 2.).abs() < 1e-14);
            if !t.boundary[i] {
                assert_eq!(t.degree(i), 6);
            }
        }
    }
}

#[test]
fn numpy_endpoint_clamping() {
    let vase = Vase::new();
    assert_eq!(vase.y(-100.), -25.);
    assert_eq!(vase.y(100.), 65.);
    assert!(vase.y(0.).abs() < 1e-12);
}
