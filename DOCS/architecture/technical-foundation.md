# ndoto technical foundation

## 1. Executive summary

`ndoto` should remain a **game-first architecture built on Rust + Bevy**, not a speculative general-purpose engine. The current prototype already proves the core differentiator: one shared world can be interpreted as **1D, 2D, 3D, and 4D (time-aware)**. The next step is to turn that visual proof into a **systemic simulation framework** where a compact set of reusable rules drives traversal, interaction, AI response, and time manipulation.

The recommended architecture is a **layered ECS-driven game runtime** with four clean boundaries:

1. **Engine layer**: app lifecycle, scheduling, rendering backend integration, asset I/O, serialization, debug tooling primitives.
2. **Game framework layer**: dimension model, world simulation, locomotion, interaction rules, time services, AI framework, quest hooks.
3. **Content/data layer**: authored prefabs, materials, interaction rules, animation graphs, level manifests, audio events, shaders, tuning configs.
4. **Tools/editor layer**: inspectors, debug overlays, replay capture, asset validation, golden scene test harnesses.

The most important architectural decisions are:

- **Single shared simulation, multiple dimensional adapters**: the world sim always lives in a canonical 3-axis space; 1D and 2D are constrained projections/adapters, not separate worlds.
- **Time as a first-class simulation service**: rewind and temporal inspection are handled by a centralized timeline system with explicit snapshot/delta policies, not by ad hoc state rewrites inside gameplay code.
- **Data-driven systemic rules**: material interactions, state propagation, audio reactions, VFX, and AI stimuli are authored as data tables and tags rather than bespoke script branches.
- **Graybox-first scope control**: the first playable version should prove movement, interaction, state propagation, and one meaningful 4D puzzle loop before advanced rendering and content scale.

This document defines the production-grade foundation for getting there while staying understandable to future coding agents.

---

## 2. Architecture principles

### 2.1 Design principles

1. **Simulation before spectacle**: world rules, interaction tags, and time integrity outrank visual fidelity.
2. **Canonical world state**: there is one source of truth for transforms, materials, status effects, and authored entities.
3. **Adapters over forks**: 1D, 2D, 3D, and 4D modes reuse the same gameplay systems whenever possible.
4. **Data over code**: designers should author interactions, surface properties, audio responses, and prefab composition through content data.
5. **Deterministic where it matters**: use fixed-step gameplay simulation and stable rule ordering to make replay, debugging, and rewind tractable.
6. **Debuggability is a feature**: every subsystem needs observable state, event traces, and golden scenes.
7. **Small-first extensibility**: start with narrow interfaces that support a graybox vertical slice; expand only after proven need.

### 2.2 Runtime assumptions

- **Language**: Rust
- **Engine/runtime base**: Bevy
- **Rendering backend**: Bevy renderer / wgpu
- **Shader language**: WGSL as canonical source format
- **Physics backend**: Jolt Physics behind a Rust adapter layer
- **Physics approach**: gameplay code talks to a dimension-aware query/character API, not directly to Jolt
- **Serialization**:
  - authoring/config: `.ron`
  - asset manifests: `.toml`
  - savegames/replays: versioned binary `CBOR` or `postcard`-style format with optional JSON export for debugging

### 2.3 Third-party dependency policy

Use dependencies only when they clearly reduce risk:

- **Keep**: `bevy`, `serde`, `ron`, `tracing`
- **Likely add**: `bevy-inspector-egui` or custom inspector UI, `serde_cbor` or `postcard`, `image`, `kira` only if Bevy audio proves insufficient for dynamic mixing needs, a maintained Jolt Rust binding or an internal `jolt-sys`/safe wrapper pair
- **Delay**: large editor frameworks, full navmesh stack, full-blown scripting language, heavy graph databases

---

## 3. Top-level subsystem map

### 3.1 Layer map

```text
+---------------------------------------------------------------+
| Tools / Editor Layer                                          |
| inspector | replay viewer | rule visualizer | asset browser   |
+---------------------------------------------------------------+
| Content / Data Layer                                          |
| prefabs | materials | rules | levels | audio events | shaders |
+---------------------------------------------------------------+
| Game Framework Layer                                          |
| dimensions | time | movement | interaction | AI | inventory   |
| world sim | quests/hooks | gameplay abilities | save schema   |
+---------------------------------------------------------------+
| Engine Layer                                                  |
| app loop | ECS schedule | renderer | asset IO | jobs | debug  |
| serialization | input | streaming | platform abstraction      |
+---------------------------------------------------------------+
| Platform                                                      |
| desktop initially; future console/mobile boundaries isolated  |
+---------------------------------------------------------------+
```

### 3.2 Runtime subsystem map

```text
Input -> Input Buffer -> Command Mapping -> Player/AI Intent
      -> Fixed Simulation Schedule
          -> Time Service
          -> Dimension Service
          -> Character Motion
          -> Physics/Queries
          -> Interaction Rules
          -> World State Propagation
          -> AI Sensing/Decision
          -> Event Bus
      -> Presentation Extraction
          -> Animation
          -> Audio Events
          -> Render Proxies
          -> UI
      -> Render Schedule
          -> Frame Graph
          -> Post FX
          -> Debug Overlays
```

### 3.3 Module breakdown

| Module | Responsibility | Depends on |
|---|---|---|
| `engine::app` | boot, main loop, schedule setup | Bevy core |
| `engine::input` | sampling, buffering, action mapping | app |
| `engine::assets` | loading, manifests, hot reload, dependency graph | app |
| `engine::physics_backend` | Jolt integration, body handles, query bridge, debug draw hooks | app |
| `engine::save` | serialization, migrations, checkpoints | assets |
| `engine::jobs` | async work, baking, background tasks | app |
| `engine::debug` | overlays, tracing, capture, inspectors | all engine modules |
| `framework::dimension` | common spatial abstraction, mode adapters | engine |
| `framework::time` | global clock, rewind, temporal snapshots | dimension |
| `framework::movement` | locomotion, climb, fall, swim-ready model | dimension, queries |
| `framework::physics` | dimension-aware queries, contacts, gameplay collision abstraction | dimension, engine::physics_backend |
| `framework::simulation` | materials, tags, propagation, environment | time, physics |
| `framework::interaction` | interactables, abilities, inventory hooks | simulation |
| `framework::ai` | sensing, planners/state machines, schedule budget | simulation, navigation |
| `framework::animation` | state graph, animation events, root motion policy | movement |
| `framework::audio` | audio events, zones, dynamic mix, debug monitor | simulation/events |
| `framework::world` | scene/region management, streaming, spawn/despawn | assets, save |
| `game::content` | game-specific prefabs, quests, abilities, rulesets | framework |
| `game::scenes` | test scenes, puzzle scenes, encounter setups | content |
| `tools::*` | inspectors, validators, graph viewers, replay UI | all |

---

## 4. Detailed subsystem designs

### 4.1 Runtime architecture

#### Main loop

Use a **dual-rate loop**:

- **Fixed simulation tick**: 60 Hz
- **Render/update tick**: variable, uncapped or vsync-limited

Why:

- deterministic-ish gameplay and stable rewind history
- stable character controller behavior
- easier replay, tests, and sync boundaries
- rendering interpolation remains independent

#### Schedule layout

```text
Startup
  -> load boot manifest
  -> register asset types
  -> load core shaders/materials
  -> spawn initial world/scene

Per-frame variable update
  -> poll OS input
  -> update hot reload watchers
  -> process async job completions
  -> run UI/debug/editor tools

FixedTick(60 Hz)
  -> copy sampled input into frame command buffer
  -> update global time + mode state
  -> run dimension adapter updates
  -> run character/AI intent systems
  -> run motion + physics queries
  -> resolve interactions and propagation
  -> update animation state model
  -> emit audio/gameplay events
  -> record timeline snapshots/deltas
  -> produce presentation snapshots

Render frame
  -> interpolate presentation state
  -> extract visible render/audio/UI data
  -> submit frame graph
```

#### Input sampling and buffering

- Sample raw device input once per render frame.
- Convert into **action states** (`move`, `jump`, `interact`, `switch_dimension`, `rewind`, `ability_primary`).
- Push into a **ring buffer** keyed by simulation tick.
- Fixed simulation consumes buffered input for the current tick.

Benefits:

- deterministic re-sim and replay support
- future rollback/network prediction compatibility
- clean handling when render FPS differs from sim FPS

#### Event bus / messaging

Use three event classes:

1. **Immediate ECS events**: local same-frame events inside the schedule.
2. **Recorded gameplay events**: persisted in replay/timeline when they affect state or must be reproducible.
3. **Debug telemetry events**: non-authoritative traces for tooling.

Rule: **events may request state changes, but canonical state lives in components/resources**.

#### Scene / world management

- World split into **regions/cells** for streaming and save scoping.
- Scene file contains:
  - region metadata
  - static prefab instances
  - authored gameplay volumes
  - environmental defaults
- Runtime keeps:
  - `ActiveWorld`
  - `LoadedRegions`
  - `PendingRegionLoads`
  - `WorldSeed` / deterministic ids

#### Save/load model

Three save layers:

1. **Profile/meta save**: settings, unlocks, discovered codex/state
2. **World save**: active region states, persistent object deltas, quest flags
3. **Checkpoint/temporal save**: shorter-lived snapshots for rewind/restart

Persist only:

- stable entity ids
- prefab references
- changed component data
- environment clock and simulation seed

Do **not** serialize transient render-only components.

#### Job system / scheduler

Use Bevy task pools with strict boundaries:

- background asset import
- nav bake
- shader validation
- audio analysis
- region streaming prep

Jobs produce typed results consumed on main thread. No background task mutates ECS world directly.

#### Determinism considerations

Full bitwise determinism across GPUs/platforms is not the initial target. The target is **gameplay determinism inside one platform/build family**.

Rules:

- fixed timestep
- stable iteration order for rule resolution
- deterministic random streams per subsystem/region
- avoid gameplay-critical floating-point branches inside render code
- isolate non-deterministic systems from timeline authority

Jolt-specific note:

- do not rely on raw Jolt simulation state as the only source of truth for rewind/save logic
- treat Jolt as the collision and rigid-body execution backend, while authoritative gameplay snapshots live in framework-owned state
- rebuild or resync Jolt bodies from authoritative gameplay state during restore paths

#### Hot reload boundaries

Hot-reload:

- shaders
- materials
- audio event configs
- interaction rule tables
- prefab data
- UI layout

Do not hot-reload:

- core save schema
- replay binary format
- entity id generation policy

---

### 4.2 Dimension architecture

#### Core idea

Use a **canonical 3-axis spatial world** with a **dimension adapter layer**:

- **3D**: full simulation/render interpretation
- **2D**: one axis compressed for movement/query/presentation rules
- **1D**: two axes compressed; gameplay reduced to line-space interaction
- **4D**: overlays temporal inspection/manipulation on top of the current spatial mode

#### Common spatial abstraction layer

Core types:

```text
WorldTransform     = translation + rotation + scale in canonical 3D
SpatialMode        = OneD | TwoD | ThreeD
TemporalMode       = Live | Rewind | FastForward | PauseInspect
DimensionState     = { spatial_mode, temporal_mode, transition_state }
DimensionProfile   = rules for projection, collision constraints, camera, visibility
```

#### Shared transform representation

All gameplay entities use the same canonical transform:

```rust
struct WorldTransform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}
```

1D/2D do **not** replace transforms. Instead they apply:

- simulation constraints
- query projection masks
- movement constraint planes
- render compression

#### Dimension-aware physics queries

Every gameplay query accepts a `DimensionQueryContext`:

```text
raycast(ctx, origin, dir, mask)
sweep_capsule(ctx, start, end, radius, height)
overlap_volume(ctx, shape, transform)
nearest_climbable(ctx, position)
```

`ctx` provides:

- active spatial mode
- projection/compression axes
- collision filters
- query tolerance scaling

Implementation detail:

- `framework::physics` exposes gameplay-friendly queries and controller sweeps
- `engine::physics_backend` translates those into Jolt broadphase/narrowphase calls
- 1D and 2D modes are implemented by constraining query axes, collision normals, and controller intent before or after Jolt queries rather than maintaining separate physics worlds

Universal systems:

- transforms
- materials/tags
- interaction rules
- save ids
- time snapshots
- event routing

Dimension-specific systems:

- movement constraints
- camera rig
- nav representation
- some rendering passes
- some physics query reductions

#### Smooth transitions

Dimension transitions have three phases:

1. **Request**: validate if mode switch is legal
2. **Resolve**: choose target projection profile and gameplay constraints
3. **Blend**: animate camera/render compression and optionally snap constrained motion state

Rule: presentation may smoothly blend; authoritative collision mode switches on a single sim tick.

#### Time as manipulable dimension

4D is not a separate spatial world. It is a **temporal control layer** over the same state:

- inspect past states
- rewind authoritative state
- optionally create localized time domains later

Integrity rule:

- only the centralized time service can apply historical state restoration
- subsystems expose snapshot/delta serializers; they never self-rewind in isolation

#### Jolt integration strategy

Use Jolt in three layers:

1. **Rigid body world**
   - dynamic props
   - static geometry
   - triggers/volumes
2. **Query layer**
   - raycasts
   - shape casts
   - overlaps
   - contact manifold extraction
3. **Character support**
   - either Jolt character support if it fits responsiveness needs, or a custom kinematic controller that uses Jolt sweeps

Recommendation:

- start with a **custom gameplay controller using Jolt sweeps and overlaps**
- avoid tying core locomotion feel to engine-provided controller behavior too early
- use Jolt rigid bodies primarily for props, collision, triggers, and robust scene queries

This keeps character feel under game control while still benefiting from Jolt’s mature collision stack.

---

### 4.3 Rendering architecture

#### Renderer foundation

Choose **clustered forward+** as the initial renderer.

Why not deferred first:

- foliage, transparency, water, particles, and stylized materials are central
- outdoor action-adventure scenes favor forward pipelines
- simpler integration with Bevy defaults and faster iteration

Deferred can be added later for specific dense indoor scenes if profiling proves it necessary.

#### Render architecture layers

```text
Asset Shader Source -> Shader Compiler/Validator -> Shader Library
Material Assets -> Material Instances -> Render Extract
World -> Render Proxies -> Visibility/Culling -> Pass Graph
Passes: depth/shadow -> opaque -> alpha/foliage -> water -> post -> debug
```

#### Material system

Material asset contains:

- shader reference
- surface domain (`opaque`, `masked`, `transparent`, `water`, `terrain`, `foliage`)
- parameter block
- texture slots
- feature flags
- render state overrides

Example:

```ron
MaterialAsset(
    shader: "shaders/surface/standard.wgsl",
    domain: Opaque,
    textures: {
        "albedo": "textures/stone_wall_a.ktx2",
        "normal": "textures/stone_wall_a_n.ktx2",
    },
    params: {
        "roughness": Float(0.82),
        "metallic": Float(0.05),
        "tint": Vec4((0.95, 0.97, 1.0, 1.0)),
    },
    features: ["RECEIVE_SHADOWS", "USE_NORMAL_MAP"],
)
```

#### Shader asset format

Canonical source:

- `.wgsl` source files on disk
- paired optional `.shader.ron` metadata for pass tags, reflection hints, and allowed variants

Example metadata:

```ron
ShaderMeta(
    label: "standard_surface",
    passes: [ShadowCaster, ForwardOpaque, DepthOnly],
    feature_groups: {
        "surface": ["USE_NORMAL_MAP", "USE_VERTEX_COLOR"],
        "lighting": ["RECEIVE_SHADOWS", "EMISSIVE"],
    },
    specialization_constants: ["ALPHA_CUTOFF"],
)
```

#### How shaders are loaded from disk

1. Asset manifest references shader files.
2. File watcher notices create/change/remove.
3. Shader import step:
   - reads `.wgsl`
   - parses optional `.shader.ron`
   - validates syntax/reflection using Naga or Bevy shader validation path
   - computes content hash
   - registers logical shader asset id
4. Runtime shader library compiles backend-ready pipelines lazily when needed by a material/pass combination.

#### Shader variants

Avoid global preprocessor explosions. Variants are formed from:

- **pass type**
- **material domain**
- **small feature bitset**
- **specialization constants**

Variant key:

```text
ShaderVariantKey {
  shader_id,
  pass,
  domain,
  features_bitmask,
  vertex_layout_id,
}
```

Rules to avoid permutation explosion:

1. split shaders by material domain instead of one mega-shader
2. keep feature groups mutually exclusive where possible
3. use specialization constants for numeric tuning, not separate variants
4. compile variants lazily and cache by content hash
5. strip unreachable variants during cook based on referenced materials/scenes

#### How materials reference shaders

Materials reference a **logical shader asset id**, not a compiled pipeline. The renderer resolves:

```text
MaterialAsset -> ShaderAssetId -> ShaderMeta -> Pass Requirements
             -> VariantKey -> PipelineCache -> GPU Pipeline
```

#### Safe runtime asset reload

Hot reload process:

1. detect changed asset + dependencies
2. import into staging asset graph
3. validate and compile staged result
4. if successful:
   - swap asset pointer/version atomically
   - mark dependent materials/pipelines dirty
   - rebuild affected pipelines next frame
5. if failed:
   - keep previous live version
   - surface error in shader/material inspector

Never replace a live asset with an invalid compile result.

#### Platform-specific shader compilation abstraction

- Canonical shader language stays WGSL.
- Backend translation is delegated to wgpu/Naga.
- Cook step can pre-validate all target platforms and emit reflection/cache metadata.
- Platform abstraction lives in `engine::render::shader_platform`, hidden behind:

```rust
trait ShaderPlatformBackend {
    fn validate(&self, source: &ShaderSource, meta: &ShaderMeta) -> ShaderValidationReport;
    fn pipeline_key(&self, variant: &ShaderVariantKey) -> PlatformPipelineKey;
}
```

#### Shadow pipeline

Initial:

- directional sun shadows with cascades
- local point/spot shadows only where authored and budgeted
- foliage and alpha-clipped casters supported

Later:

- cached static shadow pages
- contact shadow approximation

#### Lighting model

- stylized PBR-ish lighting
- one sun/moon directional source
- local emissive and point lights
- ambient probe or simple sky lighting
- fog/atmosphere integrated into post

#### Post-processing chain

Recommended order:

```text
HDR scene
-> temporal AA or SMAA (start with simpler AA)
-> bloom
-> fog/atmosphere composite
-> color grading LUT
-> vignette if needed
-> debug overlays
```

#### Terrain / foliage / water

- Terrain: start as mesh tiles with splat materials, not full sculpt editor tech
- Foliage: GPU instancing, wind parameters, LOD clusters
- Water: dedicated material domain with depth fade, shore foam hook, reflection approximation

#### LOD and culling

- CPU frustum culling
- hierarchical region/cell culling
- impostor or simplified mesh LOD later
- occlusion culling only after profiling proves need

#### Debug overlays

- wireframe
- shadow cascade view
- overdraw estimate
- culling bounds
- dimension compression axes
- material/shader ids
- light influence volumes

---

### 4.4 Audio architecture

#### Audio foundation

Audio uses the same event-driven philosophy as rendering:

```text
Gameplay state -> Audio Events -> Audio Router -> Buses/Mix States -> Playback Voices
```

#### Core systems

- `AudioEventSystem`
- `MusicStateMachine`
- `AmbientZoneSystem`
- `SurfaceFoleySystem`
- `OcclusionReverbSystem`
- `DynamicMixSystem`
- `AudioDebugMonitor`

#### Loading and streaming

- short SFX: decoded into memory at load/cook time
- long ambience/music: streamed from compressed assets in chunks
- recommended formats:
  - SFX: `.wav` at source, cooked to platform-friendly compressed buffers
  - music/ambience: `.ogg` or `.opus`

#### How audio is triggered

Sources:

- gameplay events (`InteractionStarted`, `EntityBurning`, `DimensionSwitched`)
- animation notifies (`Footstep`, `Land`, `ClimbGrip`)
- environment systems (`RainIntensityChanged`, `TimeOfDayPhaseChanged`)

Audio never polls arbitrary gameplay state every frame unless it represents a persistent zone or parameter source.

#### Parameterized audio

Audio events accept parameters:

```text
surface = stone|wood|metal|grass
speed = 0.0..1.0
weather = dry|rain|storm
danger = 0.0..1.0
time_of_day = dawn|day|dusk|night
dimension_mode = 1d|2d|3d|4d
```

Example:

```ron
AudioEvent(
    name: "footstep",
    bus: "sfx.player",
    variations: [
        ClipRef("audio/foley/footstep_stone_01.ogg"),
        ClipRef("audio/foley/footstep_stone_02.ogg"),
    ],
    params: [
        SurfaceType,
        MovementSpeed,
        WeatherState,
    ],
    cooldown_ms: 50,
)
```

#### Music state machine

States:

- exploration
- danger suspicion
- combat pressure
- puzzle focus
- temporal manipulation
- silence / sparse ambient

Transitions are driven by weighted world/gameplay conditions, not hand-scripted per area only.

#### Zones and effects

- ambient zones
- reverb volumes
- occlusion traces for major blockers
- weather and time-of-day stems

#### Debugging audio

Provide a live monitor for:

- active events and emitters
- suppressed/cooldown events
- currently playing clips by bus
- missing asset ids
- stacked duplicate triggers
- parameter values per event

Missing or invalid audio assets show a visible debug warning and emit a fallback beep only in dev builds.

---

### 4.5 Character controller architecture

#### Design goal

Movement should feel immediate and readable, but remain separated into:

1. **Gameplay locomotion model**
2. **Animation presentation**

#### Controller model

Use a **code-driven kinematic character controller** first.

Reasons:

- stable across 1D/2D/3D mode switching
- deterministic enough for replay/tests
- easier slope, step, ledge, and climb tuning
- decoupled from animation asset quality

Implementation note:

- back the controller with Jolt shape casts, overlap tests, and contact queries
- keep controller state in gameplay ECS components
- mirror only required body/query state into Jolt

Root motion can later augment climb/vault specials, but not own base traversal.

#### Locomotion states

- grounded walk
- sprint
- jump ascent
- fall
- landing
- climb hang
- climb move
- glide-ready placeholder
- swim-ready placeholder

#### Core movement stack

```text
Input Intent
-> camera-relative desired move vector
-> dimension constraint adapter
-> locomotion state machine
-> kinematic solver
-> contact resolution
-> gameplay motion result
-> animation state update
```

#### Slope / step / ledge handling

- sweep capsule with configurable step height
- classify ground by normal and material
- snap to walkable ground within tolerance
- detect ledge loss and transition to fall
- separate visual root from collision root

#### Climbing

Climbability is data-driven:

- material/tag based (`climbable`)
- optional authored climb volumes for graybox
- stamina hook later, not required at first

#### Camera-relative input

Input intent converts stick/keys into local move axes using a camera anchor, then dimension adapters constrain:

- 3D: full planar camera-relative motion
- 2D: side plane or lane plane
- 1D: signed line direction only

#### Animation architecture

Start with:

- state graph
- blend spaces for locomotion speed/direction
- animation events/notifies
- additive layers for upper body / reactions

Do **not** start with motion matching. Add it only if animation scale later demands it.

#### Multiplayer-ready foundation

Even if multiplayer is not planned immediately:

- keep input buffering
- separate predicted movement model from presentation
- use snapshot-friendly controller state

That makes later prediction/interpolation feasible.

---

### 4.6 World simulation architecture

#### Goal

Support emergent outcomes from reusable rules:

- fire spreads based on material + wind + wetness
- metal conducts electricity
- rain changes climbability, soundscape, and propagation
- AI and player respond to the same burning/wet/noisy world

#### Simulation layers

1. **Static authored properties**
   - material class
   - structural role
   - climbable/fragile/interactable flags
2. **Dynamic state tags**
   - burning
   - wet
   - frozen
   - electrified
   - moving
   - unstable
3. **Global environment state**
   - weather
   - temperature
   - time of day
   - wind vector
4. **Propagation services**
   - heat
   - moisture
   - charge
   - force / disturbance

#### Material model

Base material archetypes:

- wood
- stone
- metal
- cloth
- vegetation
- water
- earth
- glass
- flesh

Each material defines:

- conductivity
- flammability
- absorbency
- friction
- hardness
- buoyancy response
- footstep audio set
- VFX hooks

#### State tags

Represent persistent/stacking effects as explicit components:

```rust
struct Wetness { amount: f32, drying_rate: f32 }
struct Burning { intensity: f32, fuel_remaining: f32 }
struct Frozen { strength: f32 }
struct Electrified { charge: f32, ttl_ticks: u16 }
```

#### Interaction rules matrix

Use data tables, not hardcoded switch ladders.

Example:

```ron
InteractionRule(
    when: [
        HasMaterial(Wood),
        HasState(Burning),
        NearbyState(Vegetation, Dry),
    ],
    effects: [
        AddHeat(radius: 2.5, amount: 0.8),
        TryApplyState(target: Nearby, state: Burning, chance: 0.35),
    ],
    priority: 40,
)
```

#### Propagation

Run propagation in fixed-step phases:

1. gather sources
2. apply attenuation through materials/space
3. generate candidate state transitions
4. resolve deterministically by priority
5. emit gameplay/audio/VFX events

#### Authoring tools

Designers need:

- rule table editor
- material/property inspector
- volume authoring for weather/heat/sound/time
- preview scene for propagation

#### AI and player parity

The player and AI should use the same world facts:

- slippery means slippery for both
- sound stimulus created by same event pipeline
- burning hazard damages both

No “AI-only fake simulation” unless performance forces a proxy model far away from the player.

---

### 4.7 Gameplay architecture

#### Layer separation

```text
Engine layer      -> reusable technical primitives
Framework layer   -> gameplay-agnostic systemic building blocks
Content layer     -> ndoto-specific powers, quests, scenes, tuning
Tools layer       -> debugging and authoring utilities
```

#### Interfaces / contracts

Model these as data + ECS markers + systems, not heavy OOP inheritance.

Key contracts:

- `Interactable`
- `InventoryItem`
- `Equipable`
- `AbilitySource`
- `QuestHookEmitter`
- `AiAgent`
- `PhysicsBody`
- `TimeReactive`

Example:

```rust
struct Interactable {
    prompt: LocalizedTextKey,
    interaction: InteractionKind,
    allowed_modes: DimensionMask,
}

struct TimeReactive {
    snapshot_policy: SnapshotPolicy,
    temporal_flags: TemporalFlags,
}
```

#### Ability architecture

Abilities/runes/powers use:

- input trigger
- target query
- validation rules
- effect script/data graph
- cooldown/resource cost
- temporal replay policy

Keep effects data-driven where possible, with a small number of code-backed effect primitives.

---

### 4.8 AI architecture

#### Foundation

AI should be systemic-reactive, not cinematic-first.

Layers:

1. sensing
2. knowledge/memory
3. high-level state
4. action selection
5. locomotion/action execution

#### Core behavior states

- idle
- patrol/explore
- investigate
- alert
- combat
- flee/panic
- schedule/task state (day/night routines)

#### Sensory model

- sight cones and visibility checks
- sound events with falloff, material occlusion, and significance
- proximity triggers
- environment cues (fire, weather, light level, dangerous surfaces)

#### Navigation

Start simple:

- navigation volumes / grids / authored links in graybox scenes
- climb links and jump links later

Do not block the first playable milestone on a perfect open-world navmesh system.

#### Reactive world behavior

AI should react to:

- burning objects
- rain reducing visibility/noise profile
- rewound object states
- dimension changes affecting accessible paths

#### Update budgeting

NPC updates should scale with distance/importance:

- full rate near player
- reduced sensing/decision farther away
- sleep when outside relevant simulation bubble

#### Debug visualization

- sight cones
- heard sound events
- chosen goal/action
- nav path
- schedule state
- current perceived hazards

---

### 4.9 Time / 4D architecture

#### Core model

Time is both:

1. **global simulation clock**
2. **player-manipulable gameplay dimension**

#### Time layers

- `GlobalClock`: day/night, weather schedules, AI routines
- `SimTickClock`: authoritative fixed-step tick index
- `TimelineHistory`: rewindable gameplay state history
- `LocalTemporalContext`: optional future extension for special volumes/objects

#### Snapshot strategy

Use **hybrid snapshots**:

- full snapshots at coarse intervals
- per-tick deltas in between
- subsystem-level snapshot policies

Snapshot classes:

- `Always`: player, critical interactables, puzzle state
- `DeltaOnly`: moving props, environmental values
- `CheckpointOnly`: large static systems
- `Derived`: VFX, some audio, temporary presentation state

#### Temporal state deltas

Each rewindable subsystem implements:

```rust
trait TemporalState {
    type FullSnapshot;
    type Delta;

    fn capture_full(&self) -> Self::FullSnapshot;
    fn capture_delta(&self, previous: &Self::FullSnapshot) -> Option<Self::Delta>;
    fn restore_full(&mut self, snap: &Self::FullSnapshot);
    fn apply_delta_reverse(&mut self, delta: &Self::Delta);
}
```

#### Replay / rewind feasibility

Full freeform rewind for everything forever is too expensive. Constrain it:

- first target: 20-30 seconds rewind window
- limited tracked entity classes
- deterministic fixed-step event record
- no arbitrary simulation branch trees at first

#### Time-aware animation / audio / VFX

Presentation should mostly be **re-derived** from restored gameplay state.

- animation state graph can resync from locomotion state
- audio uses reversible/stoppable event policies
- VFX can respawn from state tags rather than storing every particle

#### Local time volumes

Allowed later, but only under strict rules:

- local temporal rate multiplier
- local rewind on tagged objects only
- isolated snapshot scopes

Do not add localized time until global rewind and save/load are already reliable.

#### Constraints for tractability

1. only tagged systems participate in rewind
2. fixed maximum history window
3. stable entity ids
4. snapshot policies declared per component/entity class
5. time restoration performed by central scheduler phase

---

### 4.10 Asset pipeline plan

#### Asset types

- meshes: glTF source, cooked mesh blobs
- animations: glTF clips, retarget metadata
- textures: source png/tga/exr -> cooked KTX2 or GPU-friendly compressed output
- materials: `.material.ron`
- shaders: `.wgsl` + optional `.shader.ron`
- audio: wav/ogg/opus sources -> cooked banks/streams
- collision: authored simple meshes or generated convex/heightfield metadata
- gameplay metadata: `.ron`

#### Build pipeline

```text
Source Assets
-> Importers
-> Validation
-> Dependency Graph
-> Cooked Asset Cache
-> Build Manifest
-> Runtime Asset Loader
```

Physics cooking:

- collision source meshes are imported separately from render meshes
- cook simplified convex hulls, triangle meshes, and height fields into Jolt-ready blobs
- record source hash + cooking settings in the asset manifest
- allow author-authored collision overrides per prefab

#### Asset manifests

Each build emits:

- asset id
- source hash
- dependency list
- cooked artifact paths
- version/importer revision

#### Dependency tracking

Examples:

- material depends on shader + textures
- prefab depends on mesh/material/audio/gameplay metadata
- scene depends on prefab + zone configs + nav data

#### Incremental rebuilds

On asset change:

- hash changed asset
- invalidate dependents recursively
- rebuild only affected artifacts

#### Versioning / migration

Every authored schema has:

- `schema_version`
- migration path
- validation rules

Older authored assets are upgraded during import, not by runtime branching everywhere.

#### Editor metadata

Keep editor-only metadata separate from runtime-critical data where possible.

#### Cooking / packing

Release build packs:

- cooked assets into bundles by scene/region
- optional shared core bundle
- compressed streamable chunk files

Development build keeps loose files + hot reload support.

---

### 4.11 Tooling / debug architecture

Essential tools:

- in-engine inspector
- entity/component debugger
- physics/debug query visualizer
- interaction rule visualizer
- shader/material inspector
- audio event monitor
- frame timing profiler
- asset dependency viewer
- replay capture and scrubber
- golden test scenes launcher
- Jolt body/query inspector

Tooling architecture rule:

- tools read authoritative state through debug APIs/resources
- tools never directly mutate live gameplay state outside explicit editor commands

Physics tooling should expose:

- collision shapes
- sleeping/active bodies
- contact pairs
- query hits
- broadphase bounds
- controller support normals

Golden scenes to keep permanently:

1. movement stairs/slopes scene
2. climb scene
3. material propagation scene
4. dimension transition scene
5. rewind puzzle scene
6. audio occlusion scene
7. lighting/shader validation scene

---

## 5. Data flow diagrams and schemas

### 5.1 Frame update data flow

```text
Raw Input
-> Action Mapper
-> Tick Command Buffer
-> Fixed Simulation
   -> Dimension Context
   -> Character / AI Intent
   -> Physics Queries
   -> Interaction Rules
   -> State Propagation
   -> Timeline Record
   -> Event Emit
-> Presentation State
   -> Animation
   -> Audio
   -> Render Extract
-> GPU / Audio Device
```

### 5.2 Rewind data flow

```text
Player presses rewind
-> TemporalMode = Rewind
-> Freeze forward authority
-> Read timeline history window
-> Restore subsystem snapshots/deltas in central temporal phase
-> Re-derive animation/audio/VFX from restored gameplay state
-> Present scrubbed world
-> On release:
   -> resume Live mode from restored authoritative state
```

### 5.3 Hot reload data flow

```text
File watcher detects asset change
-> importer validates source
-> dependency graph resolves affected assets
-> staged compile/cook
-> success? yes -> atomic asset version swap
               no -> keep previous live asset + surface error
-> dependent pipelines/materials/audio events marked dirty
```

### 5.4 Example entity/component model

```rust
// identity
struct StableId(Uuid);
struct PrefabRef(AssetId<PrefabAsset>);

// transform / dimension
struct WorldTransform { translation: Vec3, rotation: Quat, scale: Vec3 }
struct DimensionParticipation { allowed_modes: DimensionMask }
struct ProjectionConstraint { movement_axes: AxisMask, query_axes: AxisMask }

// physics / movement
struct KinematicBody { radius: f32, height: f32, step_height: f32 }
struct Velocity { linear: Vec3 }
struct GroundState { grounded: bool, normal: Vec3 }
struct ClimbState { active: bool, surface_entity: Option<Entity> }

// simulation
struct MaterialKind(MaterialArchetype);
struct SurfaceProperties { friction: f32, conductivity: f32, flammability: f32 }
struct Wetness { amount: f32, drying_rate: f32 }
struct Burning { intensity: f32, fuel_remaining: f32 }

// gameplay
struct Health { current: f32, max: f32 }
struct Interactable { interaction: InteractionKind }
struct InventoryItem { item_id: ItemId }
struct AbilitySet { equipped: SmallVec<[AbilityId; 4]> }

// time
struct TimeReactive { policy: SnapshotPolicy }
struct TemporalAnchor { rewind_scope: TemporalScope }

// presentation
struct AnimationStateRef { graph: AssetId<AnimGraphAsset>, state: AnimStateId }
struct AudioEmitterSet { profile: AudioProfileId }
struct RenderProxyRef { mesh: AssetId<Mesh>, material: AssetId<MaterialAsset> }
```

### 5.5 Example gameplay rule schema

```ron
RuleSetAsset(
    rules: [
        InteractionRule(
            id: "fire_spreads_to_dry_vegetation",
            when: [
                SourceHasState(Burning),
                TargetHasMaterial(Vegetation),
                TargetStateLessThan(Wetness, 0.2),
            ],
            effects: [
                AddState(Burning, intensity: 0.4),
                EmitEvent("veg_ignite"),
            ],
            cooldown_ticks: 10,
            priority: 50,
        ),
    ],
)
```

### 5.6 Example save schema

```ron
SaveGame(
    schema_version: 3,
    world_seed: 88421,
    active_region: "graybox_plateau",
    sim_tick: 203948,
    environment: EnvironmentSave(
        time_of_day_minutes: 1120,
        weather_state: LightRain,
    ),
    entity_deltas: [
        EntitySave(
            stable_id: "crate_001",
            components: {
                "WorldTransform": "...",
                "Burning": "...",
            },
        ),
    ],
)
```

---

## 6. Initial repo layout

```text
ndoto/
  Cargo.toml
  README.md
  DOCS/
    architecture/
      technical-foundation.md
    gameplay/
    systems/
    rendering/
    tooling/
  assets/
    boot/
      boot_manifest.toml
    shaders/
      surface/
        standard.wgsl
        standard.shader.ron
      terrain/
      water/
      debug/
    materials/
    meshes/
    animations/
    audio/
      events/
      music/
      ambience/
      foley/
    prefabs/
    scenes/
    rules/
    ui/
  crates/
    ndoto_engine/
      src/
        app/
        input/
        assets/
        physics_backend/
        render/
        audio/
        jobs/
        save/
        debug/
    ndoto_framework/
      src/
        dimension/
        time/
        physics/
        movement/
        simulation/
        interaction/
        animation/
        ai/
        world/
    ndoto_game/
      src/
        content/
        abilities/
        quests/
        scenes/
        bootstrap.rs
    ndoto_tools/
      src/
        inspector/
        replay/
        asset_viewer/
        rule_debug/
  src/
    main.rs
  tests/
    golden_scenes/
    integration/
  tools/
    asset_baker/
    schema_migrate/
    replay_diff/
  .github/
    workflows/
      ci.yml
```

### 6.1 Starter code skeleton

```rust
// src/main.rs
use bevy::prelude::*;
use ndoto_game::bootstrap::NdotoGamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NdotoGamePlugin)
        .run();
}
```

```rust
// crates/ndoto_game/src/bootstrap.rs
use bevy::prelude::*;
use ndoto_engine::EngineCorePlugin;
use ndoto_framework::FrameworkPlugin;

pub struct NdotoGamePlugin;

impl Plugin for NdotoGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EngineCorePlugin, FrameworkPlugin))
            .add_plugins((
                crate::content::ContentPlugin,
                crate::scenes::SceneBootstrapPlugin,
            ));
    }
}
```

```rust
// crates/ndoto_framework/src/lib.rs
use bevy::prelude::*;

pub mod ai;
pub mod animation;
pub mod dimension;
pub mod interaction;
pub mod movement;
pub mod physics;
pub mod simulation;
pub mod time;
pub mod world;

pub struct FrameworkPlugin;

impl Plugin for FrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            dimension::DimensionPlugin,
            time::TimePlugin,
            physics::PhysicsPlugin,
            movement::MovementPlugin,
            simulation::SimulationPlugin,
            interaction::InteractionPlugin,
            animation::AnimationPlugin,
            ai::AiPlugin,
            world::WorldPlugin,
        ));
    }
}
```

```rust
// crates/ndoto_framework/src/dimension/mod.rs
use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DimensionState {
    pub spatial: SpatialMode,
    pub temporal: TemporalMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpatialMode { OneD, TwoD, ThreeD }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalMode { Live, Rewind, FastForward, PauseInspect }

pub struct DimensionPlugin;

impl Plugin for DimensionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DimensionState>();
    }
}

impl Default for DimensionState {
    fn default() -> Self {
        Self { spatial: SpatialMode::ThreeD, temporal: TemporalMode::Live }
    }
}
```

```rust
// crates/ndoto_framework/src/time/temporal.rs
pub trait TemporalState {
    type FullSnapshot;
    type Delta;

    fn capture_full(&self) -> Self::FullSnapshot;
    fn capture_delta(&self, previous: &Self::FullSnapshot) -> Option<Self::Delta>;
    fn restore_full(&mut self, snapshot: &Self::FullSnapshot);
    fn apply_delta_reverse(&mut self, delta: &Self::Delta);
}
```

### 6.2 Engine/framework/game boundaries

- `ndoto_engine`: technical runtime primitives
- `ndoto_framework`: reusable gameplay systems
- `ndoto_game`: project-specific rules/content bootstrap
- `ndoto_tools`: optional dev-only utilities

---

## 7. Milestone roadmap

### 7.1 What to prototype first in graybox

First playable proof should be:

- one graybox region
- one responsive player controller
- one interactable object family
- one systemic propagation loop (for example wet/fire)
- one meaningful dimension-switch puzzle
- one rewind window applied to player + props + puzzle state

### 7.2 What to delay

Delay until after the first playable slice:

- advanced foliage tech
- complex streaming world
- local time bubbles
- motion matching
- full dynamic weather simulation
- broad quest framework
- large-scale navmesh generation

### 7.3 First 10 milestones

1. **Core schedule split**
   - fixed tick + render tick
   - input buffer
   - dimension state resource
   - tests: schedule ordering, buffered input consumption
2. **Canonical transform + dimension adapter**
   - shared transform
   - 1D/2D/3D query and movement constraints
   - tests: projection consistency, legal transitions
3. **Kinematic controller v1**
   - walk, jump, slope, ledge, step
   - tests: stairs, slopes, coyote edge, landing
4. **Interaction + material model**
   - material archetypes, state tags, rule evaluation
   - tests: deterministic rule resolution, serialization
5. **Temporal core v1**
   - fixed window snapshot/delta rewind for core entities
   - tests: restore correctness, deterministic replay, save/load integrity
6. **Presentation bridge**
   - animation state graph, render proxy extraction, audio event bus
   - tests: event emission counts, missing asset diagnostics
7. **Graybox systemic scene**
   - climbable surfaces, fire/wet puzzle, dimension switching challenge
   - tests: golden scene scripted walkthrough
8. **AI reactive agent v1**
   - patrol, investigate sound/fire, simple combat placeholder
   - tests: sensory response, budget throttling
9. **Asset pipeline + hot reload**
   - manifests, dependency graph, live reload for shader/material/rules
   - tests: reload success/failure fallback, dependency invalidation
10. **Tooling pack v1**
   - inspector, rule visualizer, replay scrubber, frame profiler
   - tests: tool smoke tests, replay diff against golden capture

### 7.4 Tests to write before major implementation

#### Runtime

- fixed tick drift and catch-up behavior
- input buffering across low/high render FPS
- event ordering across schedule sets
- Jolt world resync after restore

#### Dimension system

- 3D -> 2D -> 1D transition preserves canonical transform
- dimension-specific query masks produce expected hits
- invalid transitions are rejected deterministically
- Jolt-backed sweeps respect 1D/2D axis constraints

#### Movement

- slope walkability classification
- step-up and edge-drop behavior
- climb attach/detach correctness
- jump buffering / coyote-time if added
- controller contact resolution remains stable across Jolt broadphase updates

#### Simulation

- rule resolution order
- propagation cutoff thresholds
- material interaction table coverage

#### Time

- snapshot round-trip correctness
- rewind + resume produces valid forward state
- save/load during temporal mode is rejected or normalized
- Jolt bodies rebuild cleanly from restored authoritative state

#### Rendering/assets

- shader compile validation
- hot reload fallback on bad shader
- material parameter layout compatibility

#### Audio

- missing event id diagnostics
- duplicate/cooldown suppression
- parameterized routing by surface/weather/time

#### AI

- sensory priority resolution
- schedule transitions by time of day
- budgeted update degradation without broken state

### 7.5 Commit strategy

- one subsystem boundary per commit
- infrastructure commit before content commit
- add tests in same commit as subsystem behavior when possible
- use golden scene updates in isolated commits for easier review

### 7.6 CI recommendations

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- asset schema validation pass
- golden scene replay diff in headless mode
- shader import/validation smoke test

---

## 8. Risks and mitigation

| Risk | Why it matters | Mitigation |
|---|---|---|
| Dimension architecture forks into separate games | duplicated logic, high maintenance | keep canonical transforms and sim rules; only adapt queries/presentation |
| Rewind scope becomes unbounded | memory and debugging explode | fixed window, explicit snapshot policies, tracked entity whitelist |
| Rendering ambition outruns gameplay | prototype stalls | lock first playable slice before advanced renderer work |
| Jolt integration leaks raw engine details into gameplay | dimension switching becomes awkward and hard to replace/test | keep `framework::physics` as the only gameplay-facing API |
| Jolt FFI/binding maintenance cost | platform issues and harder upgrades | isolate bindings in `engine::physics_backend`, wrap unsafe calls in narrow safe types, add smoke tests per platform |
| Data-driven rules become opaque | hard to debug and tune | build rule visualizer and deterministic trace logs early |
| Hot reload causes unstable live state | iteration becomes unreliable | stage/validate before swap, preserve last-known-good asset |
| AI relies on special-case hacks | breaks systemic promise | AI consumes same events/tags/materials as player systems |
| Tooling postponed too long | bugs become invisible | ship inspector, replay, and golden scenes by milestone 10 at latest |

---

## 9. Starter pseudocode / implementation notes

### 9.1 Fixed tick driver

```rust
fn advance_fixed_ticks(
    time_accumulator: &mut f32,
    delta_seconds: f32,
    fixed_step: f32,
    world: &mut World,
) {
    *time_accumulator += delta_seconds;

    while *time_accumulator >= fixed_step {
        world.resource_mut::<SimTick>().0 += 1;
        world.run_schedule(FixedUpdate);
        *time_accumulator -= fixed_step;
    }
}
```

### 9.2 Rule evaluation skeleton

```rust
fn evaluate_interaction_rules(
    mut commands: Commands,
    rules: Res<RuleSetAsset>,
    query: Query<(Entity, &MaterialKind, Option<&Burning>, Option<&Wetness>)>,
) {
    for (entity, material, burning, wetness) in &query {
        for rule in rules.rules.iter().filter(|r| r.matches(material, burning, wetness)) {
            rule.apply(entity, &mut commands);
        }
    }
}
```

### 9.3 Temporal phase ordering

```text
FixedTick:
  input
  -> dimension updates
  -> temporal mode resolve
  -> if Live:
       gameplay sim
       timeline record
     else:
       timeline restore/scrub
       suppress forward-authority writes from non-temporal systems
  -> presentation extract
```

### 9.4 Recommended first slice

Build this before expanding scope:

- player can traverse a graybox scene in 3D, 2D, 1D
- one gate/puzzle requires dimension switching
- one hazard interaction (fire + wetness, or electricity + metal/water)
- one moving object affected by rewind
- one AI observer that reacts to noise and hazard changes

If that slice is stable, the architecture is working.

### 9.5 Jolt wrapper shape

```rust
pub trait PhysicsWorld {
    type BodyHandle: Copy + Send + Sync + 'static;

    fn cast_shape(&self, request: &ShapeCastRequest) -> Option<ShapeHit>;
    fn raycast(&self, request: &RayCastRequest) -> Option<RayHit>;
    fn overlap(&self, request: &OverlapRequest, out: &mut Vec<Entity>);
    fn sync_body_from_transform(&mut self, entity: Entity, transform: &WorldTransform);
    fn fetch_contacts(&self, entity: Entity, out: &mut Vec<ContactPoint>);
}
```

`engine::physics_backend` implements this trait with Jolt. `framework::movement` and `framework::simulation` depend only on the trait and the dimension query context.
