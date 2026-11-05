use sha2::{Digest, Sha256};
use vstone_core::{generate_json, model::*};
const INPUT: &str = include_str!("../../tests/fixtures/core_input_cylinder.json");
const OUTPUT: &str = include_str!("../../tests/fixtures/core_output_ring12.json");
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}
fn apply(m: [[f64; 4]; 4], p: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| (0..3).map(|j| m[i][j] * p[j]).sum::<f64>() + m[i][3])
}
#[test]
fn roundtrip_and_schema() {
    let i: CoreInput = serde_json::from_str(INPUT).unwrap();
    let o: CoreOutput = serde_json::from_str(OUTPUT).unwrap();
    assert_eq!(serde_json::to_string(&i).unwrap(), INPUT);
    assert_eq!(serde_json::to_string(&o).unwrap(), OUTPUT);
    for (data, schema) in [
        (
            INPUT,
            include_str!("../../docs/schema/core_input.schema.json"),
        ),
        (
            OUTPUT,
            include_str!("../../docs/schema/core_output.schema.json"),
        ),
    ] {
        let schema = serde_json::from_str(schema).unwrap();
        let validator = jsonschema::draft202012::new(&schema).unwrap();
        assert!(validator.is_valid(&serde_json::from_str(data).unwrap()));
    }
    assert_eq!(i.surface.profile.len(), 121);
    for (k, p) in i.surface.profile.iter().enumerate() {
        assert_eq!(*p, [9., k as f64 * 0.5]);
    }
}
#[test]
fn deterministic_and_errors() {
    assert_eq!(
        Sha256::digest(generate_json(INPUT)),
        Sha256::digest(generate_json(INPUT))
    );
    let o: CoreOutput = serde_json::from_str(&generate_json(INPUT)).unwrap();
    assert_eq!(o.status, "error");
    assert_eq!(o.message, "not implemented");
    let o: CoreOutput = serde_json::from_str(&generate_json("{bad")).unwrap();
    assert_eq!(o.status, "error");
}
#[test]
fn ring_geometry_and_nesting() {
    let o: CoreOutput = serde_json::from_str(OUTPUT).unwrap();
    assert_eq!(o.pieces.len(), 12);
    assert_eq!(o.blanks.len(), 4);
    assert!(o.validation.passed);
    assert_eq!(o.validation.checks.len(), 9);
    assert!(o
        .validation
        .checks
        .iter()
        .all(|c| c.passed && c.failures.is_empty()));
    let mut boxes = vec![];
    for p in &o.pieces {
        assert_eq!(p.sides, 4);
        assert!(p.partial);
        near(p.plane.d, 9. * (15_f64.to_radians()).cos());
        assert!((p.plane.d - 8.693).abs() < 1e-3);
        let n = p.plane.n;
        let mut pts = vec![];
        for k in 0..4 {
            let a = p.outer[k];
            let b = p.outer[(k + 1) % 4];
            let c = p.inner[(k + 1) % 4];
            let e = &p.edges[k];
            near(dot(n, a), p.plane.d);
            near(dot(n, p.inner[k]), p.plane.d - 0.375);
            near(a[0].hypot(a[2]), 9.);
            near(dot(cross(sub(b, a), sub(c, a)), sub(p.inner[k], a)), 0.);
            assert!(dot(cross(sub(b, a), sub(p.outer[(k + 2) % 4], b)), n) > 0.);
            near(dot(e.side_normal, e.side_normal), 1.);
            near(dot(e.side_normal, sub(b, a)), 0.);
            near(dot(e.side_normal, sub(c, a)), 0.);
            near(dot(sub(b, a), sub(b, a)).sqrt(), e.length);
            assert_eq!(e.from, k);
            assert_eq!(e.to, (k + 1) % 4);
            near(dot(e.side_normal, n).abs().asin().to_degrees(), e.bevel_deg);
            if e.kind == "joint" {
                near(e.bevel_deg, 15.);
                let other = o
                    .pieces
                    .iter()
                    .find(|x| Some(&x.id) == e.neighbor.as_ref())
                    .unwrap();
                let edge = other
                    .edges
                    .iter()
                    .find(|x| x.neighbor.as_ref() == Some(&p.id))
                    .unwrap();
                assert_eq!(a, other.outer[edge.to]);
                assert_eq!(b, other.outer[edge.from]);
                assert_eq!(p.inner[k], other.inner[edge.to]);
                assert_eq!(c, other.inner[edge.from]);
            } else {
                near(e.bevel_deg, 0.);
                assert!(e.neighbor.is_none());
                assert!((e.length - 4.659).abs() < 1e-3);
            }
            let f = apply(p.flat_xform, a);
            near(f[2], 0.);
            let g = apply(p.flat_xform, p.inner[k]);
            near(g[2], -0.375);
            pts.push(apply(p.blank_xform, f));
            pts.push(apply(p.blank_xform, g));
        }
        let centroid = std::array::from_fn(|i| p.outer.iter().map(|p| p[i]).sum::<f64>() / 4.);
        for x in apply(p.flat_xform, centroid) {
            near(x, 0.);
        }
        let first = sub(
            apply(p.flat_xform, p.outer[1]),
            apply(p.flat_xform, p.outer[0]),
        );
        near(first[0], 6.);
        near(first[1], 0.);
        let bounds: [f64; 4] = [
            pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min),
            pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min),
            pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max),
            pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max),
        ];
        let ox = (p.blank - 1) as f64 * 14.;
        assert!(
            bounds[0] >= ox + 0.25 - 1e-9
                && bounds[2] <= ox + 11.75 + 1e-9
                && bounds[1] >= 0.25 - 1e-9
                && bounds[3] <= 11.75 + 1e-9
        );
        boxes.push((p.blank, bounds));
    }
    for (i, (b, a)) in boxes.iter().enumerate() {
        for (bb, c) in boxes.iter().skip(i + 1) {
            if b == bb {
                let dx = (a[0] - c[2]).max(c[0] - a[2]).max(0.);
                let dy = (a[1] - c[3]).max(c[1] - a[3]).max(0.);
                assert!(dx.hypot(dy) >= 0.25 - 1e-9);
            }
        }
    }
    for b in &o.blanks {
        assert_eq!(b.pieces.len(), 3);
        assert_eq!(
            b.pieces,
            o.pieces
                .iter()
                .filter(|p| p.blank == b.index)
                .map(|p| p.id.clone())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn ring_input_provenance() {
    let raw = include_str!("../../tests/fixtures/core_input_ring12.json");
    let input: CoreInput = serde_json::from_str(raw).unwrap();
    assert_eq!(serde_json::to_string(&input).unwrap(), raw);
    let schema =
        serde_json::from_str(include_str!("../../docs/schema/core_input.schema.json")).unwrap();
    assert!(jsonschema::draft202012::new(&schema)
        .unwrap()
        .is_valid(&serde_json::from_str(raw).unwrap()));
    let output: CoreOutput = serde_json::from_str(OUTPUT).unwrap();
    assert_eq!(
        output.provenance.input_sha256,
        format!("{:x}", Sha256::digest(raw))
    );
}
