# Gates — checks only Jerry can do

Agents: stop at a gate, prepare the files listed, and wait for `Approved` written below.

## G0 — Feasibility messages (after C2 + S2)

Prepare: `vstone check` output for cylinder, cone 9→6, vase bulge, hourglass, each with `[6]`
and `[5,6,7]`, tight and wide — one text file `runs/G0/feasibility.txt`.

Jerry checks: do the messages read right, are the suggestions the ones he'd pick?

Result:

## G1 — Reference cylinder in SolidWorks (after I1)

Prepare: run folders for the cylinder at randomness 0 and 0.5 (seed 42).

Jerry checks:
- `assembled.step` opens without repair; the faceted look is what he wants; the tilt zigzag is acceptable
- `flat_layout.step`: pieces on blanks, numbers readable, nothing outside the usable square
- `assembly_map.pdf` and `report.pdf` make sense

On approval, I1 records the golden hashes.

Result:
