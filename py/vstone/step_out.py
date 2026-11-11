"""Write core-supplied inch polyhedra as labelled assembled STEP solids."""

import re
from io import BytesIO
from pathlib import Path

from build123d import Compound, Face, Shell, Solid, Unit, Wire, export_step


def piece_solid(piece):
    """Sew supplied planar faces in inches; raise ValueError for invalid solids."""
    outer, inner = piece["outer"], piece["inner"]
    if len(outer) < 3 or len(inner) != len(outer):
        raise ValueError(f"{piece['id']}: mismatched face vertices")
    polygons = [outer, list(reversed(inner))]
    for index in range(len(outer)):
        following = (index + 1) % len(outer)
        polygons.append(
            [outer[index], outer[following], inner[following], inner[index]]
        )
    faces = [Face(Wire.make_polygon(vertices, close=True)) for vertices in polygons]
    solid = Solid(Shell(faces), label=piece["id"])
    if not solid.is_valid or solid.volume <= 0:
        raise ValueError(f"{piece['id']}: faces do not form a valid closed solid")
    return solid


def write_assembled(core_output, path):
    """Write id-ordered solids in inches with a reproducible STEP timestamp."""
    solids = [
        piece_solid(piece)
        for piece in sorted(core_output["pieces"], key=lambda piece: piece["id"])
    ]
    assembly = Compound(children=solids, label="assembled")
    stream = BytesIO()
    if not export_step(assembly, stream, unit=Unit.IN, timestamp="1970-01-01T00:00:00"):
        raise OSError(f"Cannot write assembled STEP: {path}")
    occurrence = iter(range(1, len(solids) + 1))
    content = re.sub(
        rb"(NEXT_ASSEMBLY_USAGE_OCCURRENCE\(')\d+(')",
        lambda match: match[1] + str(next(occurrence)).encode() + match[2],
        stream.getvalue(),
    )
    Path(path).write_bytes(content)
