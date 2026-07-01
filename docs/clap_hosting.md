<!--
Author: Jeff
Date: 2026-06-30
Description: Historical CLAP host notes retained for archaeology only
Notes: Active architecture is VST3-only external hosting; do not implement CLAP work here
-->

# CLAP Hosting — Shelved

This document is intentionally retained as a marker that CLAP hosting is not active in the current plan.

The active external plugin boundary is `crates/geist-vst-host` and is documented in `docs/vst_hosting.md`. First-party Geist devices are internal native devices and are not exported as CLAP, VST, AU, LV2, or standalone plugin binaries.

`crates/geist-clap-host` is excluded from the workspace as historical scaffolding. Do not add features there unless Jeff explicitly changes the plugin policy.
