use vstone_core::{model::CoreInput, seeds, sidecount, surface::Surface, topology::Topology};
fn main() {
    for name in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        let mut i: CoreInput = serde_json::from_str(include_str!(
            "../../tests/fixtures/core_input_cylinder.json"
        ))
        .unwrap();
        i.surface.profile = serde_json::from_str(
            &std::fs::read_to_string(format!("../tests/fixtures/profiles/{name}.json")).unwrap(),
        )
        .unwrap();
        i.params.target_size = 6.;
        i.params.allowed_sides = vec![5, 6, 7];
        let s = Surface::from_profile(&i.surface.profile).unwrap();
        let mut rng = seeds::rng(i.params.seed);
        let t = Topology::build(seeds::place(&s, &i.params, &mut rng).unwrap()).unwrap();
        println!("PROFILE {name}");
        for j in 0..t.tiles.len() {
            if sidecount::interior(&t, j, &s) && t.degree(j) != 6 {
                println!(
                    "defect {j}: degree={} row={} s={:.6}",
                    t.degree(j),
                    t.tiles[j].row,
                    t.tiles[j].s
                );
            }
        }
        for a in 0..t.tiles.len() {
            if !sidecount::interior(&t, a, &s) {
                continue;
            }
            for &b in &t.neighbors[a] {
                if b <= a || !sidecount::interior(&t, b, &s) {
                    continue;
                }
                let ratio = seeds::spacing(&s, &t.tiles[a], &t.tiles[b]) / i.params.target_size;
                let excess = (ratio - 1.).abs() - i.params.size_variation;
                if excess > 1e-9 {
                    let exception = excess <= 0.05 + 1e-9
                        && (matches!(t.degree(a), 5 | 7) || matches!(t.degree(b), 5 | 7));
                    println!("{a}-{} degree={}/{} row={}/{} s={:.6}/{:.6} u={:.6}/{:.6} ratio={ratio:.9} {}",b,t.degree(a),t.degree(b),t.tiles[a].row,t.tiles[b].row,t.tiles[a].s,t.tiles[b].s,t.tiles[a].u,t.tiles[b].u,if exception{"valid defect exception"}else{"VIOLATION"});
                }
            }
        }
    }
}
