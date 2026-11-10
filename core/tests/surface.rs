use std::f64::consts::PI;
use vstone_core::surface::{Surface, SurfaceError};

fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b} (tol {tol})");
}

#[test]
fn cylinder_queries_and_conformal_inverse() {
    let s = Surface::from_profile(&[[9., 0.], [9., 60.]]).unwrap();
    near(s.length(), 60., 1e-12);
    assert_eq!(s.y_range(), [0., 60.]);
    for i in 0..=120 {
        let x = i as f64 * 0.5;
        near(s.gaussian_curvature(x), 0., 1e-9);
        assert_eq!(s.normal(0., x), [1., 0., 0.]);
        near(s.s_of_v(s.v_of_s(x)), x, 1e-9);
    }
    near(s.v_of_s(60.), 60. / 9., 1e-9);
    near(s.signed_distance([10., 30., 0.]), 1., 1e-12);
    near(s.signed_distance([8., 30., 0.]), -1., 1e-12);
    let (theta, arc) = s.closest_point([0., 30., -10.]);
    near(theta, 1.5 * PI, 1e-12);
    near(arc, 30., 1e-12);
    near(s.point(theta, arc)[2], -9., 1e-12);
    near(s.signed_distance([9., 62., 0.]), 2., 1e-12);
}

#[test]
fn cone_is_developable_with_outward_up_normal() {
    let s = Surface::from_profile(&[[9., 0.], [6., 60.]]).unwrap();
    for i in 1..100 {
        let x = s.length() * i as f64 / 100.;
        near(s.gaussian_curvature(x), 0., 1e-9);
        let n = s.normal(0., x);
        assert!(n[0] > 0. && n[1] > 0.);
        near(n[0].hypot(n[1]), 1., 1e-12);
        near(s.s_of_v(s.v_of_s(x)), x, 1e-9);
    }
}

#[test]
fn sphere_curvature_and_integrated_band() {
    let p: Vec<_> = (0..=160)
        .map(|i| {
            let a = (i as f64 * 0.5).to_radians();
            [10. * a.cos(), 10. * a.sin()]
        })
        .collect();
    let s = Surface::from_profile(&p).unwrap();
    for i in 5..95 {
        near(
            s.gaussian_curvature(s.length() * i as f64 / 100.),
            0.01,
            0.0002,
        );
    }
    for fraction in [0.25, 0.5, 0.9] {
        let end = s.length() * fraction;
        let h = end / 10000.;
        let integral: f64 = (0..10000)
            .map(|i| {
                let x = (i as f64 + 0.5) * h;
                s.gaussian_curvature(x) * 2. * PI * s.r(x) * h
            })
            .sum();
        near(integral, s.total_curvature(0., end), integral.abs() * 0.01);
    }
}

#[test]
fn hourglass_has_negative_waist_curvature() {
    let p: Vec<_> = (0..=100)
        .map(|i| {
            let y = i as f64 / 10.;
            [4. + (y - 5.).powi(2) * 0.1, y]
        })
        .collect();
    let s = Surface::from_profile(&p).unwrap();
    assert!(s.gaussian_curvature(s.length() / 2.) < 0.);
}

#[test]
fn rejects_invalid_profiles_with_specific_errors() {
    assert_eq!(
        Surface::from_profile(&[]).unwrap_err(),
        SurfaceError::TooFewPoints
    );
    assert_eq!(
        Surface::from_profile(&[[1., 0.]]).unwrap_err(),
        SurfaceError::TooFewPoints
    );
    for r in [0., -1., 1e-9] {
        assert_eq!(
            Surface::from_profile(&[[r, 0.], [1., 1.]]).unwrap_err(),
            SurfaceError::AxisIntersection
        );
    }
    assert_eq!(
        Surface::from_profile(&[[1., 0.], [2., 1.], [1., 0.]]).unwrap_err(),
        SurfaceError::ClosedCurve
    );
    assert_eq!(
        Surface::from_profile(&[[1., 0.], [3., 2.], [1., 2.], [3., 0.]]).unwrap_err(),
        SurfaceError::SelfIntersection
    );
    assert_eq!(
        Surface::from_profile(&[[1., 0.], [3., 0.], [2., 0.]]).unwrap_err(),
        SurfaceError::SelfIntersection
    );
    assert_eq!(
        Surface::from_profile(&[[1., 0.], [f64::NAN, 1.]]).unwrap_err(),
        SurfaceError::NonFinitePoint
    );
    assert_eq!(
        Surface::from_profile(&[[1., 0.], [1., 0.]]).unwrap_err(),
        SurfaceError::ClosedCurve
    );
}

#[test]
fn reversing_profile_is_identical() {
    let p = [[9., 0.], [8., 2.], [7., 5.], [8., 8.]];
    let a = Surface::from_profile(&p).unwrap();
    let mut reversed = p;
    reversed.reverse();
    let b = Surface::from_profile(&reversed).unwrap();
    for i in 0..=100 {
        let s = a.length() * i as f64 / 100.;
        assert_eq!(a.point(0.7, s), b.point(0.7, s));
        assert_eq!(a.normal(0.7, s), b.normal(0.7, s));
        assert_eq!(a.gaussian_curvature(s), b.gaussian_curvature(s));
        assert_eq!(a.v_of_s(s), b.v_of_s(s));
    }
}

#[test]
fn duplicates_clamping_overhang_and_distance() {
    let s = Surface::from_profile(&[[2., 0.], [2., 0.], [3., 3.], [4., 2.]]).unwrap();
    assert_eq!(s.y_range(), [0., 3.]);
    assert_eq!(s.point(0., -1.), [2., 0., 0.]);
    assert_eq!(s.point(0., s.length() + 1.), [4., 2., 0.]);
    near(s.s_of_v(-1.), 0., 0.);
    near(s.s_of_v(100.), s.length(), 0.);
    let p = s.point(0.4, s.length() * 0.75);
    near(s.signed_distance(p), 0., 1e-12);
}

#[test]
fn cone_conformal_matches_analytic_integral() {
    let surface = Surface::from_profile(&[[9., 0.], [6., 60.]]).unwrap();
    let slope = -3. / surface.length();
    for i in 0..=100 {
        let s = surface.length() * i as f64 / 100.;
        near(surface.v_of_s(s), (surface.r(s) / 9.).ln() / slope, 1e-12);
        near(surface.s_of_v(surface.v_of_s(s)), s, 1e-10);
    }
}

#[test]
fn nonadjacent_touch_and_overlap_are_rejected() {
    for p in [
        vec![[1., 0.], [3., 0.], [3., 2.], [2., 0.], [2., 3.]],
        vec![[1., 0.], [4., 0.], [4., 2.], [2., 0.], [3., 0.]],
    ] {
        assert_eq!(
            Surface::from_profile(&p).unwrap_err(),
            SurfaceError::SelfIntersection
        );
    }
}
