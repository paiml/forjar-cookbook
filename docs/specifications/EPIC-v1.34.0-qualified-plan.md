# EPIC E1 — Qualified for forjar v1.34.0

**Train:** v1.34.0 (forjar's next minor after v1.33.0). **Following train:** v1.35.0 (E2).

## Goal

The cookbook master commit that forjar records for v1.34.0 builds against
forjar 1.34.0 and every recipe validates under it. The cookbook creates no tag:
its release is that recorded commit (forjar's `cookbook_floor` rule).

## Rows

| Row | Item | done_when | Baseline (measured 2026-10-10) | First-green proof |
|-----|------|-----------|--------------------------------|-------------------|
| C1 | PMAT-017 qualify against forjar v1.34.0 | `cargo tree -p forjar --depth 0 --locked \| grep -q "forjar v1.34.0"` and `make validate-recipes` | master a8e758e locks forjar 1.32.0; forjar v1.34.0 not yet tagged | CI `gate` green on the bump PR |
| C2 | PMAT-006..016 close-out (recipes 88-98) | each `recipes/<NN>-*.yaml` validates under forjar 1.34.0, then the row is `completed` | issues #6-#16 closed, all 11 recipe files on master, rows still planned/inprogress | `make validate-recipes` on the C1 commit |
| C3 | Record the commit | forjar `docs/roadmaps/releases.yaml` v1.34.0 row names the C1 commit under `cookbook:` | releases.yaml `next:` still v1.33.0, no v1.33.0 row | forjar release PR |

## Release gate

- Only issues labelled `must-carry` block the train; anything else open on the
  `v1.34.0` milestone moves to v1.35.0 if E2 cites it, otherwise to backlog.
- forjar's clean-room is green on exactly forjar's v1.34.0 tagged commit before
  any upload (forjar's gate; the cookbook adds none of its own).
- Cookbook CI `gate` is green on the recorded commit.
- This repo never creates, moves or deletes a tag.

## Look-ahead (E2, v1.35.0)

One slot, kit at `docs/lookahead/v1.35.0.yaml`. It proposes tickets through the
cop inbox and never mints them. During a release pass it yields CI capacity and
opens no PR.
