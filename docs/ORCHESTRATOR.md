# Vorona — orchestrator handoff (for a new Claude chat)

Written by Claude on 2026-11-18 to carry the project from a long chat into a new one. Read this file,
then `AGENTS.md`, `amendments/A04`–`A06` (A06 incl. §8 and §9), and the task card in progress.

## 1. Who does what
- **Jerry (owner)** decides design, approves every merge, runs OpenClaw/terminal commands himself.
- **JerryClaw** = OpenClaw agent (Telegram), runs the project, commits and merges. Main model
  `openai/gpt-6.1-terra` (set 2026-11-17; fallbacks `gpt-6.1-sol`, then local `ollama/qwen3-coder`).
  Delegates Rust to the **vorona-core** subagent (`openai/gpt-6.1-sol`).
- **Claude (orchestrator, you)** reads logs and code on the Mac, reviews before every merge,
  prototypes geometry, drafts amendments and messages for JerryClaw. Jerry pastes them.
- A **Mac-side Claude reviewer** (run by JerryClaw) is a first check only; it never authorises a merge.

## 2. Rules that must not drift
- **No merge into main without Claude-in-chat review AND Jerry's explicit go-ahead.**
- All builds/tests run in the `Vorona-dev` container (`make dev-check`, `make acceptance`); never
  overlap two acceptance runs (harness now has a lock + heartbeat).
- Home Assistant containers untouched; only create/remove `Vorona-*`/`vstone-*` Docker objects, never prune.
- Ask Jerry before any OpenClaw config change; give him the exact commands.
- Never paste tokens or secrets in chat.
- **Every message for JerryClaw goes in a copyable code block** (Jerry's standing preference).
- Report in plain words; Jerry reviews pictures — render stones (top/side views) when geometry changes.

## 3. Design as of 2026-11-18 (owner decisions)
- Pattern `voronoi_triangles` (A06): one polygon per seed + a triangle at every former 3-stone corner;
  every corner shared by 4 pieces, so the shell can follow straight surfaces (no rings).
- §9 (addendum 2): small symmetric stones. Polygon and triangle sides ≤ 3 in, triangle sides ≥ 1.5 in,
  isosceles symmetric triangles, **one stone count and one row height for the whole surface** (stones
  stretch with the taper; count changes only if forced, in even steps). Symmetric relief default
  depth **0.25 in**; random relief is experimental (fails often).
- Rims: level saw cut **always at the design ends (0 and H)**; avoid slivers by **shifting the pattern
  phase**, never the cut. Saw-cut edges exempt from max edge and from edge minimums; rim partials need a
  1.5 in minimum square.
- Joints: bisector planes; near-coplanar (< 1° half-angle) use one shared plane perpendicular to the
  mean face normal. Back-face gap at 4-piece corners reported (typ. 0.07–0.09 in, max ~0.2 in).
- Thickness 3/8 in; taper (bevel) cap 60°; plan-view corner angle ≥ 30°; pieces fit 12×12 blank (11.5 usable).
- The older A05 pattern (`pattern = "voronoi"`) is kept unchanged with its own regression matrix.

## 4. State on 2026-11-18
- C5/C6 and C7 (flat transforms and conservative rectangle nesting) are merged into main. See git log for the merge commits and `tasks/C7-nesting.md` for review evidence.
- Next: **S4** flat layout output, then S5–S7, I1 and G1, stopping for review before each merge. C7b shape-aware packing is a separate, not-started follow-up.
- Known follow-ups: cylinder A05 r0.5/seed3 timeout (accepted); random relief rework. Requests 006 and 007 are tracked historical drafts; A06 §8 is the governing addendum.

## 5. How to check logs (device shell on the Mac)
Repo on the Mac: `~/Projects/Vorona/voronoi-stone`; worktrees in `runs/worktrees/<card>/`, logs in each
worktree's `runs/logs/`. Git inside worktrees fails from the linked VM (absolute /Users paths); run git from
the main folder and compare files with `cmp <(git show <commit>:<path>) <worktree>/<path>`.
```bash
cd ~/Projects/Vorona/voronoi-stone
git log --oneline -5 main; git ls-remote --heads origin; git branch --sort=-committerdate | head
ls -lt runs/worktrees/<card>/runs/logs | head
```
Acceptance summary: `acceptance-*/matrix96.stderr.log` (A05: expect pass 94 + 2 accepted timeouts) and
`acceptance-*/a06-summary.log` (A06: six required cases exit 0). Files nested more than 7 folders below
`Projects` cannot be staged to the cloud; copy a compact extract to `runs/render_tmp/` first.

## 6. Review checklist before approving a merge
1. Committed files identical to the tested working tree (provenance log lists HEAD + dirty files).
2. dev-check passes; acceptance clean (no overlapping runs; timings normal).
3. Rims exactly at 0/H; closure Ok; no dangling neighbours; flatness ≤ 1e-6; deterministic repeat.
4. Symmetric cases: rows exact copies (check independently from piece geometry).
5. No limit relaxed without an owner decision recorded in an amendment.
6. Read the key code paths, not only the logs.

## 7. OpenClaw notes
- Heartbeat and memory dreaming are switched off (they burned ChatGPT quota in the background).
- ChatGPT plan limits: 5-hour window + weekly. When exhausted, JerryClaw falls to local Qwen and
  rambles; wait for reset (`/status` shows usage) or use a banked reset. Start `/new` per card.
- JerryClaw's automatic follow-up turns lose the spawn tool; a user message restores it. It must never
  use the `openclaw` CLI to start subagents (blocked by design).
- Prototypes: `docs/prototypes/A06_tri_proto_v2.py` (reference only).
