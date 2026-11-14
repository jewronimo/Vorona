use vstone_core::{model::CoreInput, seeds, surface::Surface, topology::Topology};
fn main() {
    let i: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let s = Surface::from_profile(&i.surface.profile).unwrap();
    let sites = seeds::place(&s, &i.params, &mut seeds::rng(1)).unwrap();
    let t = Topology::build(sites).unwrap();
    let mut e = std::collections::BTreeMap::new();
    for &[a, b, c] in &t.corners {
        for x in [(a, b), (a, c), (b, c)] {
            *e.entry(x).or_insert(0) += 1;
        }
    }
    for ((a, b), n) in e {
        if n == 1 {
            println!("edge {a}-{b}: {:?} {:?}", t.tiles[a], t.tiles[b]);
        }
    }
}
