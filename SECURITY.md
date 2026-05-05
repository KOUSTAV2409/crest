# Security policy

## Reporting a vulnerability
If you believe you’ve found a security issue in Crest, please **do not** open a public issue.

Instead, email: **security@koustav.dev**.

Please include:
- What you found and why it’s impactful
- Steps to reproduce (PoC if possible)
- Your environment (distro, desktop, Wayland/Xorg)

## Scope notes
- Crest is **local-first** and does not ship telemetry.
- Extensions/plugins execute as separate processes; by default Crest runs only scripts listed in the user’s manifest file.

