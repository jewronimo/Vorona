//! Independent polygon oracle: no nesting bounds/separation helpers are reused.
use vstone_core::{flat, model::CoreOutput, nest};
type Point = [f64; 2];
fn cross(a: Point, b: Point, c: Point) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
// The specified footprint is the convex hull of BOTH projected faces.
fn hull(mut points: Vec<Point>) -> Vec<Point> {
    assert!(points.iter().flatten().all(|v| v.is_finite()));
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup();
    let mut lower = Vec::new();
    for &p in &points {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper = Vec::new();
    for &p in points.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    assert!(lower.len() >= 3);
    lower
}
fn edges(p: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    p.iter()
        .copied()
        .zip(p.iter().copied().cycle().skip(1))
        .take(p.len())
}
fn point_segment(p: Point, a: Point, b: Point) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let length2 = d[0] * d[0] + d[1] * d[1];
    let t = if length2 == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / length2).clamp(0.0, 1.0)
    };
    (p[0] - a[0] - t * d[0]).hypot(p[1] - a[1] - t * d[1])
}
fn segment_distance(a: Point, b: Point, c: Point, d: Point) -> f64 {
    let ac = cross(a, b, c);
    let ad = cross(a, b, d);
    let ca = cross(c, d, a);
    let cb = cross(c, d, b);
    if ((ac > 0.0 && ad < 0.0) || (ac < 0.0 && ad > 0.0))
        && ((ca > 0.0 && cb < 0.0) || (ca < 0.0 && cb > 0.0))
    {
        return 0.0;
    }
    point_segment(a, c, d)
        .min(point_segment(b, c, d))
        .min(point_segment(c, a, b))
        .min(point_segment(d, a, b))
}
fn polygon_distance(a: &[Point], b: &[Point]) -> f64 {
    // Hulls are CCW; containment must be checked as well as boundary distance.
    let inside = |p, polygon: &[Point]| edges(polygon).all(|(a, b)| cross(a, b, p) >= 0.0);
    if inside(a[0], b) || inside(b[0], a) {
        return 0.0;
    }
    let mut distance = f64::INFINITY;
    for (p, q) in edges(a) {
        for (r, s) in edges(b) {
            distance = distance.min(segment_distance(p, q, r, s));
        }
    }
    distance
}
fn assert_clearance(out: &CoreOutput, gap: f64) {
    let footprints: Vec<_> = out
        .pieces
        .iter()
        .map(|p| {
            hull(
                p.outer
                    .iter()
                    .chain(&p.inner)
                    .map(|v| {
                        let q = flat::apply(&p.blank_xform, flat::apply(&p.flat_xform, *v));
                        [q[0], q[1]]
                    })
                    .collect(),
            )
        })
        .collect();
    let mut pairs = 0;
    for (i, a) in out.pieces.iter().enumerate() {
        for (j, b) in out.pieces.iter().enumerate().skip(i + 1) {
            if a.blank != b.blank {
                continue;
            }
            pairs += 1;
            let distance = polygon_distance(&footprints[i], &footprints[j]);
            assert!(
                distance + 1e-9 >= gap,
                "blank {} {} / {}: polygon distance {distance} < {gap}",
                a.blank,
                a.id,
                b.id
            );
        }
    }
    assert!(pairs > 0, "clearance check must exercise same-blank pairs");
}
#[test]
fn oracle_detects_crossing_containment_touching_and_diagonal_clearance() {
    let square = hull(vec![[0., 0.], [2., 0.], [2., 2.], [0., 2.]]);
    let contained = hull(vec![[0.5, 0.5], [1.5, 0.5], [1., 1.5]]);
    assert_eq!(polygon_distance(&square, &contained), 0.0);
    let horizontal = hull(vec![[-2., -0.5], [2., -0.5], [2., 0.5], [-2., 0.5]]);
    let vertical = hull(vec![[-0.5, -2.], [0.5, -2.], [0.5, 2.], [-0.5, 2.]]);
    assert_eq!(polygon_distance(&horizontal, &vertical), 0.0);
    for (shift, expected) in [
        (2., 0.),
        (2.125, 0.125_f64 * 2_f64.sqrt()),
        (2.25, 0.25_f64 * 2_f64.sqrt()),
    ] {
        let other: Vec<_> = square
            .iter()
            .map(|v| [v[0] + shift, v[1] + shift])
            .collect();
        assert!((polygon_distance(&square, &other) - expected).abs() < 1e-12);
    }
    assert_eq!(
        segment_distance([0., 0.], [2., 0.], [1., 0.], [3., 0.]),
        0.0
    );
}
#[test]
fn packed_ring12_polygon_clearance_includes_inner_faces() {
    let mut out: CoreOutput =
        serde_json::from_str(include_str!("../../tests/fixtures/core_output_ring12.json")).unwrap();
    for p in &mut out.pieces {
        flat::flatten(p, 0.375).unwrap();
    }
    out.blanks = nest::pack(&mut out.pieces, 12., 11.5, 0.25).unwrap();
    assert_clearance(&out, 0.25);
    // Synthetic bevel overhang: inner footprint is wider than outer; do not
    // accidentally certify only the outer polygon. Existing transforms remain rigid.
    for p in &mut out.pieces {
        let n = p.plane.n;
        let center: [f64; 3] = std::array::from_fn(|k| {
            p.inner.iter().map(|v| v[k]).sum::<f64>() / p.inner.len() as f64
        });
        for v in &mut p.inner {
            let delta: [f64; 3] = std::array::from_fn(|k| v[k] - center[k]);
            let normal: f64 = (0..3).map(|k| delta[k] * n[k]).sum();
            for k in 0..3 {
                v[k] += 0.08 * (delta[k] - normal * n[k]);
            }
        }
    }
    out.blanks = nest::pack(&mut out.pieces, 12., 11.5, 0.25).unwrap();
    assert_clearance(&out, 0.25);
    // Deliberately overlap two pieces to prove the output oracle catches corruption.
    let i = 0;
    let j = (1..out.pieces.len())
        .find(|&j| out.pieces[j].blank == out.pieces[i].blank)
        .unwrap();
    out.pieces[j].blank_xform = out.pieces[i].blank_xform;
    assert!(std::panic::catch_unwind(|| assert_clearance(&out, 0.25)).is_err());
}
#[test]
#[ignore = "full production generation; run explicitly for C7 review"]
fn generated_a06_polygon_clearance() {
    let input: vstone_core::model::CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_a06_cylinder.json"
    ))
    .unwrap();
    let out = vstone_core::generate(&input);
    assert!(!out.pieces.is_empty(), "{}", out.message);
    assert!(!out.blanks.is_empty(), "{}", out.message);
    assert_clearance(&out, input.params.nest_gap);
    eprintln!(
        "production polygon clearance: {} pieces, {} blanks",
        out.pieces.len(),
        out.blanks.len()
    );
}
