# Activate the agreed paid directory using the repaired framework revision

Level: Consequential
Decided by: Shawn
Rests on: FND-001 PAY-001 PAY-016 paid-directory
Would be wrong if: The pinned repair fails the starter contracts or the implementation treats Larafast as redistributable source instead of a working behavior reference.

## Decision

The developer confirmed the remaining requirement text, falsifiers, publishing and refund policies, and exact repaired Suprnova revision 107e6e7a122d5145160ea1547ca90ddc37459c27, then corrected the working reference to larafast-directories-master rather than Pulsar and instructed implementation to proceed. Activate paid-directory first, then complete-starter, with predecessor regressions. Amend FND-001 and PAY-001 to the immutable repaired framework revision. Adapt Larafast flows and UI behavior into the existing Rust/Vue/Vuetify 0 application; do not copy purchased source or assets into the MIT distribution. This authorizes starter implementation, not another framework release gate, release tag, deployment, or new provider.

## Realized by

(none yet: recorded, not built)
