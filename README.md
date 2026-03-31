# ndoto

A dreamlike game experiment built with Rust and Bevy.

Current milestone: dimensional readability prototype.

The project uses one shared greybox scene and proves how the same space reads in:
- 3D via a perspective camera
- 2D via an orthographic camera plus depth compression
- 1D via a tighter orthographic camera plus height and depth compression

Controls:
- `1` switches to 3D
- `2` switches to 2D
- `3` switches to 1D

Scope:
- rendering and presentation only
- no gameplay, physics, combat, UI, puzzle logic, or authored assets yet
