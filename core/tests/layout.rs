use vstone_core::{flat, model::CoreOutput, nest};
fn ring() -> CoreOutput {
    serde_json::from_str(include_str!("../../tests/fixtures/core_output_ring12.json")).unwrap()
}
#[test]
fn ring_flat_and_three_per_blank_repeatably() {
    let mut out = ring();
    for piece in &mut out.pieces {
        flat::flatten(piece, 0.375).unwrap();
        for point in &piece.outer {
            assert!(flat::apply(&piece.flat_xform, *point)[2].abs() < 1e-9);
        }
        for point in &piece.inner {
            assert!((flat::apply(&piece.flat_xform, *point)[2] + 0.375).abs() < 1e-9);
        }
    }
    let blanks = nest::pack(&mut out.pieces, 12.0, 11.5, 0.25).unwrap();
    assert_eq!(blanks.len(), 4);
    assert!(blanks.iter().all(|b| b.pieces.len() == 3));
    let mut again = out.pieces.clone();
    assert_eq!(blanks, nest::pack(&mut again, 12.0, 11.5, 0.25).unwrap());
    assert_eq!(out.pieces, again);
    // The enclosing rectangles contain both faces, so rectangle separation
    // independently proves polygon clearance, including tapered inner overhang.
    let mut rectangles = Vec::new();
    for p in &out.pieces {
        let mut b = [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ];
        for point in p.outer.iter().chain(&p.inner) {
            let v = flat::apply(&p.blank_xform, flat::apply(&p.flat_xform, *point));
            b[0] = b[0].min(v[0]);
            b[1] = b[1].min(v[1]);
            b[2] = b[2].max(v[0]);
            b[3] = b[3].max(v[1]);
        }
        let offset = (p.blank - 1) as f64 * 14.0;
        assert!(b[0] >= offset + 0.25 - 1e-9 && b[2] <= offset + 11.75 + 1e-9);
        assert!(b[1] >= 0.25 - 1e-9 && b[3] <= 11.75 + 1e-9);
        for &(blank, a) in &rectangles {
            let a: [f64; 4] = a;
            if blank == p.blank {
                assert!(
                    b[2] + 0.25 <= a[0] + 1e-9
                        || a[2] + 0.25 <= b[0] + 1e-9
                        || b[3] + 0.25 <= a[1] + 1e-9
                        || a[3] + 0.25 <= b[1] + 1e-9
                );
            }
        }
        rectangles.push((p.blank, b));
    }
}
#[test]
fn invalid_faces_and_unplaceable_pieces_are_errors() {
    let mut out = ring();
    out.pieces[0].inner[0][0] += 1.0;
    assert!(flat::flatten(&mut out.pieces[0], 0.375).is_err());
    let mut out = ring();
    for p in &mut out.pieces {
        flat::flatten(p, 0.375).unwrap();
    }
    let before = out.pieces.clone();
    assert!(nest::pack(&mut out.pieces, 1.0, 1.0, 0.25).is_err());
    assert_eq!(before, out.pieces);
    assert!(nest::pack(&mut out.pieces, 12.0, 11.5, f64::NAN).is_err());
}

#[test]
fn centroid_is_area_centroid_not_vertex_average_and_longest_edge_is_positive_x() {
    let mut p = ring().pieces.remove(0);
    p.plane.n = [0.0, 0.0, 1.0];
    p.plane.d = 7.0;
    // Unequally subdividing an edge must not move the area centroid.
    p.outer = vec![
        [10.0, 20.0, 7.0],
        [16.0, 20.0, 7.0],
        [16.0, 24.0, 7.0],
        [10.0, 24.0, 7.0],
        [10.0, 23.0, 7.0],
    ];
    p.inner = p.outer.iter().map(|v| [v[0], v[1], v[2] - 0.375]).collect();
    flat::flatten(&mut p, 0.375).unwrap();
    let centroid = flat::apply(&p.flat_xform, [13.0, 22.0, 7.0]);
    assert!(centroid.iter().all(|v| v.abs() < 1e-9));
    let a = flat::apply(&p.flat_xform, p.outer[0]);
    let b = flat::apply(&p.flat_xform, p.outer[1]);
    assert!((b[0] - a[0] - 6.0).abs() < 1e-9 && (b[1] - a[1]).abs() < 1e-9);
    let m = p.flat_xform;
    let determinant = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    assert!((determinant - 1.0).abs() < 1e-9);
}
