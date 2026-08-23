# ST8WRX Upstream Strategy

ST8WRX is built on the open-source Buzz codebase from Block. The fork should remain able to absorb upstream improvements without forcing ST8WRX to inherit Buzz branding or product positioning.

## Rules

1. **Preserve upstream history.** Do not rewrite or squash the inherited Buzz history for branding purposes.
2. **Preserve license and attribution.** Apache-2.0 notices and applicable upstream copyright/NOTICE material remain intact.
3. **Prefer extension seams over invasive rewrites.** ST8WRX economic/provenance capabilities should attach to existing Projects, Mesh, Relay/Audit, agent metrics, and workflow boundaries.
4. **Keep upstream package names where renaming would create unnecessary merge churn.** Internal crate names such as `buzz-core` and `buzz-relay` may remain implementation names while the product is branded ST8WRX.
5. **Brand at product boundaries.** README, documentation, app chrome, icons, release metadata, installer identity, public deployment defaults, and user-visible copy should progressively move to ST8WRX.
6. **Never silently import upstream branding regressions.** Upstream sync review must explicitly inspect user-facing strings/assets before merge.
7. **No blockchain work on the collaboration hot path.** Buzz remains the fast signed-event workspace. ST8WRX provenance and economic settlement remain asynchronous unless a feature explicitly requires synchronous economic finality.

## Upstream source

Canonical upstream: `block/buzz`

Fork lineage should remain documented even after the repository itself is renamed.

## Sync workflow

Recommended maintenance loop:

1. fetch upstream `main`
2. compare upstream changes against ST8WRX `main`
3. classify changes as core/platform, product UI, infrastructure, or branding
4. merge reusable core/platform improvements
5. resolve branding-sensitive files deliberately
6. run full ST8WRX CI gates
7. record notable inherited capabilities in the ST8WRX changelog

## Branding boundary

The following should eventually be ST8WRX-owned:

- repository name and description
- README/front page
- desktop/mobile application names
- app icons and splash assets
- package/display metadata where externally visible
- deployment examples and public image names
- website/social imagery
- product documentation and screenshots

The following may retain Buzz-derived internal naming when changing it would provide little user value and high upstream merge cost:

- internal crate/module identifiers
- protocol implementation names
- migration history
- historical docs required to understand inherited architecture
- environment variables until a compatibility/deprecation plan exists

This separation is intentional. ST8WRX should become a distinct product without throwing away the engineering leverage of staying close to a rapidly improving upstream codebase.
