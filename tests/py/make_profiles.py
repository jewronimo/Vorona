"""Generate AP214 curve fixtures, coordinates in inches."""

from pathlib import Path

from build123d import Compound, Edge, Unit, Vector, Wire, export_step

ROOT = Path(__file__).resolve().parents[2]


def main():
    """Write valid and invalid STEP curves with deterministic headers."""
    fixtures = {
        "cylinder": Edge.make_line((9, 0, 0), (9, 60, 0)),
        "cone": Edge.make_line((9, 0, 0), (6, 60, 0)),
        "vase": Edge.make_spline([Vector(9, 0), Vector(12, 30), Vector(9, 60)]),
        "hourglass": Edge.make_spline([Vector(9, 0), Vector(6, 30), Vector(9, 60)]),
        "mixed": Wire(
            [
                Edge.make_line((9, 0, 0), (9, 30, 0)),
                Edge.make_three_point_arc((9, 30, 0), (12, 45, 0), (9, 60, 0)),
            ]
        ),
        "closed": Edge.make_circle(3),
        "crosses_axis": Edge.make_line((9, 0, 0), (-3, 60, 0)),
        "touches_axis": Edge.make_line((9, 0, 0), (0, 60, 0)),
        "two_curves": Compound(
            [
                Edge.make_line((9, 0, 0), (9, 20, 0)),
                Edge.make_line((9, 30, 0), (9, 60, 0)),
            ]
        ),
        "not_xy": Edge.make_line((9, 0, 5), (9, 60, 5)),
    }
    folder = ROOT / "tests/fixtures/step"
    folder.mkdir(parents=True, exist_ok=True)
    for name, shape in fixtures.items():
        export_step(
            shape,
            folder / f"{name}.step",
            unit=Unit.IN,
            timestamp="1970-01-01T00:00:00",
        )
    (ROOT / "examples/cylinder_18x60_profile.step").write_bytes(
        (folder / "cylinder.step").read_bytes()
    )


if __name__ == "__main__":
    main()
