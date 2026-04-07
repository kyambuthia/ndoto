# ndoto

A dreamlike game experiment built with Rust and Bevy.

The core idea is dimensional play:
- 3D space as the full world
- 2D as a compressed reading of that same world
- 1D as the most extreme spatial reduction
- 4D as time manipulation, including turning back time

Current milestone: gameplay foundation.

The current prototype established the dimensional foundation with one shared greybox scene and proved how the same space reads in:
- 3D via a perspective camera
- 2D via an orthographic camera plus depth compression
- 1D via a tighter orthographic camera plus height and depth compression
- 4D via reversible time layered over the current spatial view

The next phase is turning that foundation into an actual game:
- build core player controls and moment-to-moment interaction
- design gameplay that makes 1D, 2D, 3D, and 4D meaningfully useful
- replace the pure readability sandbox with playable greybox scenarios

Current controls:
- `1` switches to 3D
- `2` switches to 2D
- `3` switches to 1D
- `4` toggles 4D time mode on top of the current spatial view
- hold `Space` or `R` to rewind while in 4D mode
- hold `F` to fast-forward through recorded history while in 4D mode

Current scope:
- dimensional rendering and reversible time are proven prototype features
- gameplay systems, puzzle logic, and interaction design are now the active focus
- the current scene is still greybox, but it now serves as a base for gameplay iteration

Future scope:
- player abilities and game rules built around dimensional switching and time reversal
- puzzle logic, UI, authored assets, progression, and other game systems
