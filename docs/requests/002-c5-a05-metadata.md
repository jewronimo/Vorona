# C5 A05 output metadata implementation mapping

A05 already approves adding seed depth, maximum corner distance and reflex corner
count to each piece (R-OUT-4). This implementation uses `seed_depth`,
`max_corner_distance` (signed inches, positive outside) and `reflex_corners`.
Fields are optional for historical fixture compatibility, emitted for all new
C5 pieces. Both embedded schemas and INTERFACE are updated together. No input
setting or manufacturing limit changes. SHELL must consume these fields when
implementing its A05 report mirror; CORE has not edited Python.

Example: `{"seed_depth":0.75,"max_corner_distance":0.12,"reflex_corners":1}`.
Owner review before merge must confirm this mapping and selected rim cut heights.
