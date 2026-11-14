//! Targeted seam/root-cause diagnostic; does not replace parent acceptance.
use vstone_core::{assembly, closure, model::CoreInput, planes, sidecount, surface::Surface};
fn main() {
    let i: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let s = Surface::from_profile(&i.surface.profile).unwrap();
    let t = sidecount::build(&s, &i.params).unwrap();
    let g = planes::fitted(&s, &t, &i.params).unwrap();
    println!(
        "sites={} boundary={} complete_fans={} measured={} violations={:?}",
        t.tiles.len(),
        t.boundary.iter().filter(|&&x| x).count(),
        g.polygons.iter().filter(|p| p.len() >= 3).count(),
        g.whole_tiles.iter().filter(|&&x| x).count(),
        g.report.violated
    );
    match assembly::assemble(&t, &g, &s, &i.params) {
        Ok(a) => println!(
            "pieces={} removals={:?} closure={:?}",
            a.pieces.len(),
            a.rejections,
            closure::certify(&a.pieces, a.rim_heights)
        ),
        Err(e) => println!("HARD FAILURE {e}"),
    }
}
