"""Import one open STEP curve chain and sample in inches."""

import math

from build123d import Vector, Wire, import_step
from OCP.GCPnts import GCPnts_QuasiUniformDeflection


class ProfileError(ValueError):
    """Invalid STEP profile."""


def read_and_sample_profile(step_path, chord_error=0.01):
    """Return XY inch coordinates; fail on topology, plane, radius or sampling errors."""
    if not math.isfinite(chord_error) or not 0 < chord_error <= 0.01:
        raise ProfileError("chord_error must be in (0, 0.01] inches")
    try:
        model = import_step(str(step_path)).scale(1 / 25.4)
        edges = model.edges()
        if not edges or model.faces():
            raise ProfileError("Expected one open curve chain without faces")
        wires = Wire.combine(edges, tol=1e-7)
        if len(wires) != 1:
            raise ProfileError("Expected one curve chain; disconnected curves found")
        wire = wires[0]
        if wire.is_closed:
            raise ProfileError("Profile must be open, not closed")
        bounds = wire.bounding_box()
        if max(abs(bounds.min.Z), abs(bounds.max.Z)) > 1e-6:
            raise ProfileError("Profile must lie in XY plane")
        if bounds.min.X <= 1e-9:
            raise ProfileError("Profile crosses or touches the Y axis")
        points = []
        for edge in wire.order_edges():
            sampler = GCPnts_QuasiUniformDeflection(
                edge.geom_adaptor(), chord_error / 2
            )
            if not sampler.IsDone() or sampler.NbPoints() < 2:
                raise ProfileError("Curve sampling failed")
            sampled = [
                Vector(sampler.Value(index))
                for index in range(1, sampler.NbPoints() + 1)
            ]
            start = edge.start_point()
            if (start - sampled[0]).length > (start - sampled[-1]).length:
                sampled.reverse()
            points.extend([[point.X, point.Y] for point in sampled][bool(points) :])
        return points
    except (OSError, ValueError, RuntimeError) as error:
        if isinstance(error, ProfileError):
            raise
        raise ProfileError(f"Cannot read STEP profile: {error}") from error
