# GitHub publication

Private repository: https://github.com/Apopal1/macropad-studio-rust

## Verified remote commits
- `ce622ca55a4b2b35197964234d313a81af63c2ea`: repository initialization.
- `0f5c1cbb56afb46d6eb6b4a46fb3152a23f0eef5`: native source, roadmap, checklist and documentation (55 files).
- `30e4881be8bf617aeb88ab7ae8a7d2c20d69855e`: Windows/Linux CI workflow.

The GitHub connector has no installed repository access and the local GitHub CLI is unauthenticated. Publication therefore used the user's authenticated GitHub web interface. No application permission or credential changes were made. This publishes a source snapshot; it does not rewrite the original six local development commits into the GitHub history. That original history is preserved in the earlier delivery bundle and this local checkout.

For future development, authenticate a Git client for this specific private repository and clone its existing `main`. Do not force-push the original unrelated local history over GitHub `main`. The bootstrap-only `scripts/publish-github.ps1` intentionally refuses an existing repository; do not run it again for this already-created repository.

Initial CI: https://github.com/Apopal1/macropad-studio-rust/actions/runs/45412907930
At publication time it is running. The checklist and validation report separate local tests, hosted compilation and real hardware acceptance.
