"""S3 acceptance against the independent ring fixture."""

import json
from pathlib import Path

import numpy as np
import pytest
from build123d import import_step

from vstone.step_out import piece_solid, write_assembled


@pytest.fixture
def ring():
    return json.loads(
        (Path(__file__).parents[1] / "fixtures/core_output_ring12.json").read_text()
    )


def polyhedron_volume(piece):
    outer, inner = piece["outer"], piece["inner"]
    faces = [list(reversed(outer)), inner]
    faces.extend(
        [
            outer[index],
            outer[(index + 1) % len(outer)],
            inner[(index + 1) % len(outer)],
            inner[index],
        ]
        for index in range(len(outer))
    )
    return abs(
        sum(
            np.dot(face[0], np.cross(face[index], face[index + 1])) / 6
            for face in faces
            for index in range(1, len(face) - 1)
        )
    )


def test_reimport_labels_validity_and_volume(ring, tmp_path):
    path = tmp_path / "assembled.step"
    write_assembled(ring, path)
    imported = import_step(path)
    assert len(imported.solids()) == 12
    children = sorted(imported.children, key=lambda child: child.label)
    assert [child.label for child in children] == [
        f"P{index:03}" for index in range(1, 13)
    ]
    for child, piece in zip(children, ring["pieces"], strict=True):
        child = child.scale(1 / 25.4)
        assert child.is_valid
        assert child.volume == pytest.approx(polyhedron_volume(piece), rel=0.001)
        assert child.bounding_box().min == pytest.approx(
            piece_solid(piece).bounding_box().min
        )


def test_byte_identical_and_id_order(ring, tmp_path):
    first, second = tmp_path / "first.step", tmp_path / "second.step"
    write_assembled(ring, first)
    ring["pieces"].reverse()
    write_assembled(ring, second)
    assert first.read_bytes() == second.read_bytes()
    assert b"1970-01-01T00:00:00" in first.read_bytes()


def test_invalid_vertex_counts(ring):
    piece = ring["pieces"][0]
    piece["inner"].pop()
    with pytest.raises(ValueError, match="P001"):
        piece_solid(piece)
