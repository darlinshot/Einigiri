# Einigiri — Main Project Design

## 1. Project Overview

Einigiri is a Linux ricing / desktop shell framework for Wayland.

The project is conceptually similar to QuickShell, but it aims to provide a simpler and more approachable UI API inspired by Roblox:

- **Luau** is the configuration and scripting language.
- **Rust** is the framework/backend implementation language.
- **GTK4** is the initial UI toolkit/backend.
- **Wayland** is the target display protocol.
- Wayland layer-shell is used for desktop-shell surfaces such as bars, panels, overlays, widgets, notifications, etc.

The primary design goal is to let users write UI configurations such as:

```lua
local surface = Surface {
    Anchor = {
        Top = true,
        Left = true,
        Right = true
    }
}

local panel = Frame {
    Width = 500,
    Height = 100
}

local title = TextLabel {
    Text = "Einigiri",
    Width = 200,
    Hefight = 40
}

title.Parent = panel
panel.Parent = surface
```

without requiring the user to understand GTK internals, Wayland protocol details, or a large collection of framework-specific modules.

The framework should provide high-level Luau classes such as:

- `Frame`
- `TextLabel`
- `ImageLabel`
- `Button`
- `ImageButton`
- future controls/widgets

These classes should primarily be implemented at the Luau API level rather than requiring a matching Rust class for every widget.

---

# 2. Core Architectural Principle

The most important architectural principle is:

> **Do not make Luau classes map directly to GTK widgets or Wayland surfaces.**

Instead, Einigiri should have a layered architecture:

```text
                         EINIGIRI
                            │
          ┌─────────────────┴─────────────────┐
          │                                   │
        Luau                              Rust Core
          │                                   │
   ┌──────┴───────┐                ┌──────────┴────────┐
   │              │                │                   │
 Classes       User code          UI Core            Runtime
   │              │                │                   │
 Frame         init.luau        Entity Tree          Luau VM
 TextLabel     bar.luau         Components             │
 Button        widgets.luau     Properties             │
   │                               Events              │
   └──────────────┬────────────────┘                   │
                  │                                    │
                  ▼                                    │
              Layout Engine                            │
                  │                                    │
                  ▼                                    │
             Surface Manager                           │
                  │                                    │
                  ▼                                    │
             GTK Backend                               │
                  │                                    │
                  ▼                                    │
            GTK4 / GDK                                 │
                  │                                    │
                  ▼                                    │
         Wayland / layer-shell ◄───────────────────────┘
```

The responsibilities of each layer must remain clearly separated.

### Luau

Describes what the user wants.

Examples:

```lua
Frame {
    Width = 500,
    Height = 100
}

TextLabel {
    Text = "Hello"
}
```

### UI Core

Represents the user's UI as a framework-independent scene/tree.

### Layout Engine

Resolves high-level layout declarations into concrete geometry.

### Surface Manager

Manages top-level Wayland shell surfaces and their Wayland-specific properties.

### GTK Backend

Translates the framework's abstract UI representation into GTK widgets and GTK operations.

### Wayland

Provides the actual desktop/display surface through GTK/GDK and layer-shell integration.

---

# 3. Why This Architecture Is Necessary

Wayland introduces an important constraint for a shell framework.

A Wayland layer-shell surface does not behave like an arbitrary desktop window whose position can simply be set to any `(x, y)` coordinate.

Instead, a shell surface uses concepts such as:

- anchors
- margins
- layer
- exclusive zone
- monitor/output
- keyboard interaction

Therefore, Wayland geometry should **not** be exposed as the geometry system of every UI element.

For example, a `TextLabel` should not need to know that its parent is ultimately a Wayland layer-shell surface.

Instead:

```text
Wayland Surface
    │
    └── GTK root
          │
          └── Einigiri UI tree
                ├── Frame
                ├── TextLabel
                └── Button
```

The Wayland surface handles desktop-level positioning.

The UI layout engine handles the positioning of children inside that surface.

---

# 4. Two Levels of Anchoring

Einigiri should conceptually have two separate kinds of anchors.

## 4.1 Surface Anchors

Surface anchors determine how a top-level surface is positioned relative to the Wayland output.

Example:

```lua
Surface {
    Anchor = {
        Top = true,
        Left = true,
        Right = true
    }
}
```

This should eventually map to Wayland/layer-shell behavior such as:

```text
Top    = true
Left   = true
Right  = true
```

along with:

- margins
- exclusive zone
- layer
- monitor/output
- keyboard interactivity

This logic belongs to:

```text
src/surface/
src/backend/wayland/
```

or the equivalent structure chosen during implementation.

## 4.2 UI Anchors

UI anchors are part of Einigiri's own layout system.

Example:

```lua
Frame {
    Anchor = {
        Left = true,
        Right = true
    }
}
```

This means the frame is anchored to its parent, not directly to Wayland.

The layout engine converts this into concrete geometry:

```text
Parent
┌──────────────────────────────┐
│                              │
│ Frame                        │
│ ┌──────────────────────────┐ │
│ │                          │ │
│ └──────────────────────────┘ │
│                              │
└──────────────────────────────┘
```

This allows the Luau API to remain high-level and Roblox-like while still working correctly with Wayland.

---

# 5. Top-Level Surface Concept

A `Surface` should be conceptually similar to a Roblox `ScreenGui`.

Roblox-style hierarchy:

```text
PlayerGui
   │
   └── ScreenGui
         │
         ├── Frame
         ├── TextLabel
         └── ImageLabel
```

Einigiri-style hierarchy:

```text
Einigiri
   │
   └── Surface
         │
         ├── Frame
         ├── TextLabel
         └── ImageLabel
```

A `Surface` is responsible for desktop/shell concerns:

- Wayland surface creation
- output/monitor
- layer
- surface anchors
- margins
- exclusive zone
- keyboard interactivity
- surface lifecycle

Its children are responsible for UI concerns:

- layout
- styling
- text
- images
- interaction
- events

---

# 6. UI Core Architecture

The Rust UI core should use an **Entity + Components + Tree** architecture.

The framework should not create a Rust class hierarchy such as:

```rust
struct Frame { ... }
struct TextLabel { ... }
struct ImageLabel { ... }
struct Button { ... }
```

as its fundamental UI representation.

Instead, use a generic entity.

Conceptually:

```rust
pub struct Entity {
    id: EntityId,
    parent: Option<EntityId>,
}
```

An entity is given behavior/data through components.

Example:

```text
Entity #42

Components:
    Layout
    Style
    Text
    Interaction
```

Another entity might have:

```text
Entity #43

Components:
    Layout
    Style
    Image
```

This means a high-level Luau class does not need to correspond to a unique Rust type.

---

# 7. Recommended Components

The exact component API can evolve, but the initial system should be based around components such as:

```text
Layout
Style
Text
Image
Interaction
Children / hierarchy
```

### Layout

Potential data:

```text
width
height
min_width
min_height
max_width
max_height
position
anchors
margin
padding
alignment
```

### Style

Potential data:

```text
background
foreground
opacity
border
border_radius
font
font_size
```

### Text

Potential data:

```text
text
font
font_size
weight
alignment
wrapping
```

### Image

Potential data:

```text
source
size
fit
alignment
```

### Interaction

Potential data:

```text
clickable
hoverable
focusable
pointer interaction
keyboard interaction
```

The framework should avoid creating one component for every high-level Luau class unless there is a real architectural need.

---

# 8. Luau Classes vs Rust Components

This distinction is fundamental.

## Rust defines capabilities

Rust should define the low-level capabilities that the framework understands:

```text
Entity
Component
Layout
Style
Text
Image
Input
Events
Animation
```

## Luau defines user-facing classes

Luau can define:

```text
Instance
GuiObject
Frame
TextLabel
ImageLabel
Button
ImageButton
...
```

For example:

```text
Instance
   │
   └── GuiObject
         │
         ├── Frame
         ├── TextLabel
         ├── ImageLabel
         └── Button
```

The hierarchy is primarily a **Luau abstraction**.

Rust does not need to mirror this entire hierarchy.

A `TextLabel` might simply create an entity containing:

```text
Layout
Style
Text
Interaction (if necessary)
```

This makes the framework easier to extend.

Adding a new high-level widget should often require only Luau code.

---

# 9. UI Tree

The UI core must maintain a parent/child hierarchy.

Example Luau:

```lua
local panel = Frame {
    ...
}

local title = TextLabel {
    ...
}

title.Parent = panel
```

should create a tree conceptually equivalent to:

```text
Frame Entity #1
    │
    └── TextLabel Entity #2
```

The UI tree is independent of GTK's widget tree.

The GTK backend mirrors the framework tree into GTK.

Therefore:

```text
Luau UI Tree
     │
     ▼
Rust UI Tree
     │
     ▼
GTK Widget Tree
```

The GTK tree is an implementation detail.

---

# 10. Property System

The framework should have a centralized property/update mechanism.

Conceptually:

```rust
pub enum Property {
    Position(Position),
    Size(Size),
    Visible(bool),
    Opacity(f32),
    Text(String),
    Font(Font),
    Color(Color),
    BackgroundColor(Color),
}
```

The exact representation can change, but properties should be treated as framework-level state rather than direct GTK calls.

Example:

```lua
label.Text = "Hello"
```

Flow:

```text
Luau
  │
  ▼
Property update
  │
  ▼
Entity #123
  │
  ▼
Text component
  │
  ▼
GTK backend
  │
  ▼
GtkLabel
```

This abstraction will later make it easier to implement:

- animations
- bindings
- reactive updates
- hot reload
- debugging
- serialization
- transitions

---

# 11. Event System

Events should be a framework-level subsystem rather than being implemented independently inside every widget.

Luau API should eventually support patterns such as:

```lua
button.MouseEnter:Connect(function()
    ...
end)

button.MouseLeave:Connect(function()
    ...
end)

button.MouseClick:Connect(function()
    ...
end)
```

The flow should be:

```text
GTK event
    │
    ▼
GTK backend event adapter
    │
    ▼
EntityId + framework Event
    │
    ▼
Event Dispatcher
    │
    ▼
Luau callback
```

This means GTK-specific events remain inside the GTK backend.

The UI core exposes framework-level events.

---

# 12. Layout System

The layout system should be independent of GTK and Wayland.

It takes declarative UI layout information:

```text
size
position
anchors
padding
margin
alignment
constraints
```

and produces resolved geometry:

```rust
pub struct ResolvedGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

Conceptual flow:

```text
User layout specification
        │
        ▼
    Layout data
        │
        ▼
   Layout resolver
        │
        ▼
ResolvedGeometry
        │
        ▼
    GTK backend
```

This allows Einigiri's public layout API to remain independent from GTK.

---

# 13. Layout and GTK

Initially, the project should strongly consider using Einigiri's layout system to resolve geometry and then applying the resulting geometry to GTK.

Conceptually:

```text
Einigiri layout
       │
       ▼
resolved x/y/width/height
       │
       ▼
GTK container/widget allocation
```

GTK itself has its own layout and allocation mechanisms. The project should use GTK's capabilities where useful, but the **public Einigiri layout model must not become dependent on GTK's API**.

The long-term goal is:

```lua
Position = ...
Size = ...
AnchorPoint = ...
```

or an equivalent Einigiri-specific API, while Rust translates that abstraction into GTK.

---

# 14. GTK Backend

GTK4 should be treated as a backend rather than as the UI architecture itself.

The UI core should not contain GTK-specific types wherever avoidable.

Instead:

```text
src/ui/
    Entity
    Component
    Tree
    Property
    Event

src/backend/gtk/
    GTK-specific implementation
```

The GTK backend translates:

```text
Einigiri UI model
        │
        ▼
    GTK backend
        │
        ├── GtkWidget
        ├── GtkLabel
        ├── GTK containers
        ├── CSS
        └── GTK events
```

This separation also keeps the door open for a future rendering/backend implementation without redesigning the Luau API.

---

# 15. Backend Abstraction

A backend abstraction should eventually exist.

Conceptually:

```rust
pub trait UiBackend {
    type Widget;

    fn create(&mut self, entity: EntityId) -> Self::Widget;

    fn destroy(&mut self, entity: EntityId);

    fn update_property(
        &mut self,
        entity: EntityId,
        property: Property,
    );

    fn update_layout(
        &mut self,
        entity: EntityId,
        geometry: ResolvedGeometry,
    );
}
```

The initial implementation will be:

```text
UiBackend
    │
    └── GtkBackend
```

Do not implement multiple backends prematurely. The abstraction should exist only when it helps maintain the boundary between framework logic and GTK.

---

# 16. Surface Manager

The project should have a dedicated surface subsystem.

Suggested structure:

```text
src/surface/
    mod.rs
    surface.rs
    configuration.rs
    manager.rs
```

A surface conceptually contains:

```rust
pub struct Surface {
    pub id: SurfaceId,
    pub monitor: MonitorId,
    pub layer: Layer,
    pub anchors: SurfaceAnchors,
    pub margin: Margins,
    pub exclusive_zone: i32,
    pub root: EntityId,
}
```

The exact fields and types should be determined during implementation.

The surface manager is responsible for:

- creating top-level shell surfaces
- destroying surfaces
- configuring Wayland/layer-shell behavior
- associating a surface with a root UI entity
- handling output/monitor selection
- managing surface lifecycle

---

# 17. Wayland Responsibilities

Wayland-specific behavior must remain isolated from normal UI components.

Wayland/layer-shell concerns include:

```text
surface
layer
anchor
margin
exclusive zone
output
keyboard interactivity
surface lifecycle
```

Normal UI elements should not need to know about these concepts.

For example:

```text
TextLabel
    ↓
does NOT know about Wayland
```

while:

```text
Surface
    ↓
Wayland layer-shell
```

This prevents Wayland limitations from leaking throughout the framework.

---

# 18. Configuration and Luau Runtime

The current configuration-related files:

```text
src/config.rs
src/luau_resolver.rs
```

should eventually be split into more clearly defined responsibilities.

Recommended conceptual structure:

```text
src/config/
    mod.rs
    loader.rs
    resolver.rs
    environment.rs

src/luau/
    mod.rs
    runtime.rs
    bindings.rs
    events.rs
```

## `config/loader.rs`

Responsible for locating and loading user configuration.

Example:

```text
~/.config/einigiri/init.luau
```

## `config/resolver.rs`

Responsible for resolving Luau modules/imports.

## `config/environment.rs`

Responsible for constructing the environment available to configuration scripts.

## `luau/runtime.rs`

Responsible for creating and managing the Luau VM.

## `luau/bindings.rs`

Responsible for exposing Rust framework functionality to Luau.

Potential exposed concepts:

```text
Surface
Instance
Color
Vector2
Size
Position
events
```

## `luau/events.rs`

Responsible for connecting framework events to Luau callbacks.

---

# 19. Recommended Project Structure

The project should move toward a structure similar to:

```text
einigiri/
│
├── Cargo.toml
├── Cargo.lock
│
├── src/
│   │
│   ├── main.rs
│   ├── einigiri.rs
│   │
│   ├── app/
│   │   ├── mod.rs
│   │   ├── runtime.rs
│   │   └── lifecycle.rs
│   │
│   ├── config/
│   │   ├── mod.rs
│   │   ├── loader.rs
│   │   ├── resolver.rs
│   │   └── environment.rs
│   │
│   ├── luau/
│   │   ├── mod.rs
│   │   ├── runtime.rs
│   │   ├── bindings.rs
│   │   └── events.rs
│   │
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── entity.rs
│   │   ├── tree.rs
│   │   ├── component.rs
│   │   ├── property.rs
│   │   ├── event.rs
│   │   └── registry.rs
│   │
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── geometry.rs
│   │   ├── anchor.rs
│   │   ├── resolver.rs
│   │   └── constraints.rs
│   │
│   ├── surface/
│   │   ├── mod.rs
│   │   ├── surface.rs
│   │   ├── configuration.rs
│   │   └── manager.rs
│   │
│   ├── backend/
│   │   ├── mod.rs
│   │   │
│   │   ├── gtk/
│   │   │   ├── mod.rs
│   │   │   ├── renderer.rs
│   │   │   ├── widgets.rs
│   │   │   ├── events.rs
│   │   │   └── style.rs
│   │   │
│   │   └── wayland/
│   │       ├── mod.rs
│   │       ├── surface.rs
│   │       └── layer_shell.rs
│   │
│   └── error.rs
│
├── luau/
│   ├── core.luau
│   │
│   ├── classes/
│   │   ├── Instance.luau
│   │   ├── GuiObject.luau
│   │   ├── Frame.luau
│   │   ├── TextLabel.luau
│   │   ├── ImageLabel.luau
│   │   └── Button.luau
│   │
│   └── libraries/
│       ├── color.luau
│       ├── geometry.luau
│       └── animation.luau
│
├── examples/
│   ├── bar.luau
│   ├── notification.luau
│   └── launcher.luau
│
└── target/
```

The exact module names may change during implementation, but the architectural boundaries should remain.

---

# 20. Module Responsibilities

The following responsibilities should guide future implementation.

## `app`

Application-level orchestration.

Responsible for:

- startup
- initialization order
- runtime lifecycle
- shutdown
- coordinating major subsystems

It should not contain widget-specific behavior.

## `config`

User configuration loading and module resolution.

It should not contain UI implementation.

## `luau`

Luau VM integration and Rust-to-Luau bindings.

It should not contain GTK implementation.

## `ui`

Framework-independent UI representation.

Responsible for:

- entities
- components
- hierarchy
- properties
- events
- entity/component registration

It should not depend directly on GTK or Wayland.

## `layout`

Framework-independent layout calculations.

Responsible for:

- layout rules
- anchors
- constraints
- geometry calculation

It should not directly create GTK widgets.

## `surface`

Top-level shell surface abstraction.

Responsible for:

- surface lifecycle
- surface configuration
- connecting a root UI entity to a top-level surface

## `backend/gtk`

GTK-specific rendering and widget management.

## `backend/wayland`

Wayland/layer-shell-specific functionality.

---

# 21. Startup Flow

The application should conceptually start like this:

```text
main()
  │
  ▼
Einigiri::new()
  │
  ├── initialize GTK
  │
  ├── initialize Wayland/layer-shell integration
  │
  ├── initialize UI manager
  │
  └── initialize Luau runtime
          │
          ▼
      load configuration
          │
          ▼
      execute init.luau
          │
          ▼
      Luau creates objects
          │
          ▼
      Rust Entity tree
          │
          ▼
      Layout resolution
          │
          ▼
      GTK widgets created
          │
          ▼
      GTK hierarchy created
          │
          ▼
      Wayland surface configured
          │
          ▼
      event loop
```

The exact initialization order may change based on GTK and Wayland requirements.

---

# 22. Detailed UI Creation Flow

Given:

```lua
local panel = Frame {
    width = 500,
    height = 100,
    backgroundColor = Color.rgb(20, 20, 20)
}

local title = TextLabel {
    text = "Hello",
    parent = panel
}

panel.Parent = surface
```

the conceptual flow is:

```text
                        Luau
                          │
                          ▼
                   Frame.new(...)
                          │
                          ▼
                    Rust binding
                          │
                          ▼
                     Entity #1
                          │
                ┌─────────┼─────────┐
                ▼         ▼         ▼
             Layout     Style    Children
                │         │
                │         │
                ▼         ▼
           width=500   background
           height=100
                          │
                          ▼
                   TextLabel.new()
                          │
                          ▼
                     Entity #2
                          │
                          ▼
                    Parent = #1
                          │
                          ▼
                       UI Tree
                         #1
                          │
                          └── #2
                              │
                              ▼
                       Layout Engine
                              │
                              ▼
                     Resolved Geometry
                              │
                              ▼
                        GTK Backend
                              │
                         ┌────┴────┐
                         ▼         ▼
                    GTK widget  GtkLabel
                         │         │
                         └────┬────┘
                              ▼
                         GTK Window
                              │
                              ▼
                       Wayland Surface
```

---

# 23. Event Flow

Example:

```lua
button.MouseClick:Connect(function()
    print("clicked")
end)
```

Flow:

```text
Wayland / GTK
      │
      ▼
GTK input event
      │
      ▼
GTK backend event adapter
      │
      ▼
EntityId + Event
      │
      ▼
UI Event Dispatcher
      │
      ▼
Luau callback
```

The event system should avoid exposing raw GTK events to users.

---

# 24. Styling Philosophy

One of Einigiri's goals is to avoid requiring users to import many modules simply to style UI.

The public API should therefore prefer:

```lua
TextLabel {
    text = "Hello",
    fontSize = 16,
    color = Color.white
}
```

rather than forcing users to deal with:

```lua
require(...)
require(...)
require(...)
GTK CSS...
```

The internal implementation may still use GTK CSS or other GTK mechanisms.

The distinction is:

```text
User API
    ↓
simple, declarative, Luau-oriented

Internal implementation
    ↓
GTK CSS / GTK properties / GTK widgets
```

Users should not need to know how the styling is implemented.

---

# 25. Roblox-Inspired API Philosophy

Einigiri should borrow the **simplicity and object model**, not necessarily Roblox's exact implementation.

Desired characteristics:

```text
Instance
GuiObject
Parent
Children
Properties
Events
Classes
```

Example:

```lua
local panel = Frame {
    Name = "Panel",
    Size = ...,
    Position = ...
}

local title = TextLabel {
    Name = "Title",
    Text = "Hello"
}

title.Parent = panel
```

The goal is to make UI configuration feel like manipulating a tree of objects rather than constructing GTK widgets manually.

---

# 26. Important Boundary: UI Elements Are Not Wayland Surfaces

A major implementation rule:

> **Do not create one Wayland surface per UI element.**

The desired architecture is:

```text
Wayland surface
    │
    └── GTK root
          │
          └── Frame
                ├── TextLabel
                ├── ImageLabel
                └── Button
```

not:

```text
Wayland surface
 ├── Frame surface
 ├── TextLabel surface
 ├── Button surface
 └── Image surface
```

Most UI elements should exist as widgets within a top-level surface.

---

# 27. Initial Vertical Slice

Before implementing the entire framework, the first milestone should be a minimal complete path:

```text
Luau
  ↓
Surface
  ↓
Frame
  ↓
Entity
  ↓
Layout
  ↓
GTK
  ↓
Wayland
```

The first functional example should resemble:

```lua
local surface = Surface {
    anchor = {
        top = true,
        left = true,
        right = true
    }
}

local panel = Frame {
    width = 500,
    height = 100
}

local title = TextLabel {
    text = "Einigiri",
    width = 200,
    height = 40
}

title.Parent = panel
panel.Parent = surface
```

Expected conceptual representation:

```text
Surface #0
│
└── Frame #1
      │
      └── TextLabel #2
```

with:

```text
Surface #0
    ↓
Wayland layer surface

Frame #1
    ↓
GTK container

TextLabel #2
    ↓
GtkLabel
```

Do not attempt to implement the entire widget system before this vertical slice works.

---

# 28. Initial Implementation Priorities

Implementation should proceed approximately in this order:

## Phase 1 — Application/runtime foundation

Implement:

- application lifecycle
- GTK initialization
- basic Wayland/layer-shell integration
- Luau VM initialization
- configuration loading

## Phase 2 — UI Core

Implement:

- `EntityId`
- entity storage
- parent/child tree
- component storage
- basic properties

## Phase 3 — Luau bindings

Implement:

- entity creation
- property access
- parent assignment
- basic object/class support

## Phase 4 — Surface

Implement:

- top-level surface abstraction
- surface anchors
- output selection
- layer
- margins
- exclusive zone

## Phase 5 — Layout

Implement:

- size
- position
- anchors
- resolved geometry
- basic parent-child layout

## Phase 6 — GTK backend

Implement:

- entity → GTK widget mapping
- widget creation
- hierarchy synchronization
- property synchronization
- geometry synchronization

## Phase 7 — First widgets

Implement Luau classes:

```text
Frame
TextLabel
ImageLabel
Button
```

Only implement additional Rust functionality when required by these classes.

## Phase 8 — Events

Implement:

- click
- hover
- pointer events
- keyboard/focus events where needed

## Phase 9 — Styling

Implement the high-level style/property API and GTK translation.

---

# 29. Features to Avoid Implementing Prematurely

Do not initially build:

```text
Animation system
Reactive signal system
Full ECS framework
Multiple rendering backends
Complex constraint solver
Plugin system
Hot reload
Advanced CSS abstraction
Large widget library
```

These can be added after the core architecture is stable.

The first objective is a complete, understandable vertical slice.

---

# 30. Future Extensibility

The architecture should eventually allow features such as:

```text
Animations
Reactive properties
Data binding
Hot reload
Multiple surfaces
Multiple monitors
Notifications
Desktop widgets
Bars
Panels
Launchers
OS integration
Custom widgets
Theming
Reusable Luau libraries
```

without requiring major changes to the core architecture.

For example:

```text
Animation
    ↓
Property system
    ↓
Component
    ↓
Backend
```

rather than implementing animation separately inside every widget.

---

# 31. Architectural Rules for Future Development

AI agents and developers working on the project should follow these rules.

### Rule 1

**Do not couple Luau classes directly to GTK classes.**

### Rule 2

**Do not couple normal UI components directly to Wayland.**

### Rule 3

**Keep Wayland surface concerns in the surface/Wayland layer.**

### Rule 4

**Keep GTK-specific implementation inside the GTK backend.**

### Rule 5

**Keep the UI core framework-independent.**

### Rule 6

**Prefer generic entities/components over a large Rust widget class hierarchy.**

### Rule 7

**Prefer Luau classes for high-level widget abstractions.**

### Rule 8

**The public API should optimize for simplicity, not for exposing GTK internals.**

### Rule 9

**The layout system must be independent from Wayland surface positioning.**

### Rule 10

**Do not add abstractions merely for theoretical extensibility. Add them when they establish a useful boundary or solve a real problem.**

### Rule 11

**Avoid implementing a feature in multiple layers when one layer should own it.**

For example:

```text
Wayland surface anchors → Surface/Wayland
UI anchors              → Layout
GTK widget creation     → GTK backend
Widget class definition → Luau
Entity representation   → UI core
```

### Rule 12

**Preserve the conceptual flow:**

```text
Luau
  ↓
UI model
  ↓
Layout
  ↓
Surface/backend
  ↓
GTK
  ↓
Wayland
```

---

# 32. AI Agent Development Strategy

This document is the **main architectural design document**.

It should not contain every implementation detail.

Instead, it should be used as the source of truth from which more specific implementation documents are generated.

Future AI-agent instructions should be split into separate Markdown files.

For example:

```text
docs/
├── architecture.md              # This document
│
├── modules/
│   ├── app.md
│   ├── config.md
│   ├── luau-runtime.md
│   ├── ui-core.md
│   ├── entity.md
│   ├── components.md
│   ├── property-system.md
│   ├── event-system.md
│   ├── layout.md
│   ├── surface.md
│   ├── gtk-backend.md
│   └── wayland-backend.md
│
├── luau/
│   ├── object-model.md
│   ├── instance.md
│   ├── gui-object.md
│   ├── frame.md
│   ├── text-label.md
│   ├── image-label.md
│   └── button.md
│
└── implementation/
    ├── phase-01-runtime.md
    ├── phase-02-ui-core.md
    ├── phase-03-luau-bindings.md
    └── phase-04-layout.md
```

The exact documentation structure can evolve.

The important principle is:

```text
architecture.md
      │
      ├── defines WHAT the project should be
      │
      ▼
module-specific MD files
      │
      ├── define HOW each module should be implemented
      │
      ▼
AI coding agent
      │
      ▼
Rust / Luau implementation
```

---

# 33. How Future AI Agents Should Use This Document

When an AI agent is asked to implement a module, it should first use this document to understand:

1. The overall architecture.
2. The responsibilities of the requested module.
3. Which modules it may depend on.
4. Which modules must remain independent.
5. The direction of data flow.
6. The distinction between Luau, UI core, layout, GTK, and Wayland.

Then a more specific implementation MD file should provide:

```text
Purpose
Responsibilities
Public API
Data structures
Dependencies
Data flow
Interaction with other modules
Invariants
Error handling
Tests
Implementation tasks
Non-goals
```

An implementation agent should not redesign the entire architecture while implementing one module unless the architecture is demonstrably impossible or contradictory.

If a design conflict is discovered, the agent should identify the conflict explicitly rather than silently introducing a new architecture.

---

# 34. Overall Data Flow

The complete intended flow is:

```text
                       USER CONFIG
                           │
                           ▼
                         Luau
                           │
                  creates/modifies objects
                           │
                           ▼
                    Luau bindings
                           │
                           ▼
                    UI Entity Tree
                           │
                 ┌─────────┴─────────┐
                 │                   │
                 ▼                   ▼
             Components          Properties
                 │                   │
                 └─────────┬─────────┘
                           │
                           ▼
                      Layout Engine
                           │
                           ▼
                  Resolved Geometry
                           │
                           ▼
                    Surface Manager
                           │
                           ▼
                      GTK Backend
                           │
                           ▼
                       GTK4/GDK
                           │
                           ▼
                Wayland / layer-shell
```

Events travel in the opposite direction:

```text
Wayland
   │
   ▼
GTK
   │
   ▼
GTK Backend
   │
   ▼
UI Event System
   │
   ▼
Luau callback
```

---

# 35. Final Design Summary

Einigiri should be designed around the following model:

```text
LUau
────
High-level user-facing object/class API.

UI Core
───────
Framework-independent entity/component/tree representation.

Layout
──────
Converts declarative UI layout into resolved geometry.

Surface
───────
Represents top-level Wayland shell surfaces.

GTK Backend
───────────
Translates the abstract UI representation into GTK4.

Wayland Backend
───────────────
Handles Wayland/layer-shell-specific behavior.
```

The central relationship is:

```text
                Luau Classes
                     │
                     ▼
              Generic Entities
                     │
                Components
                     │
                     ▼
               Layout Engine
                     │
                     ▼
              Resolved Geometry
                     │
                     ▼
                GTK Backend
                     │
                     ▼
             Wayland Surface
```

The framework should feel simple to the user:

```lua
TextLabel {
    text = "Hello",
    fontSize = 16
}
```

while internally remaining strongly separated:

```text
Luau API
   ≠
Rust UI Model
   ≠
Layout System
   ≠
GTK
   ≠
Wayland
```

This separation is the foundation of Einigiri's architecture and should be preserved as the project grows.
