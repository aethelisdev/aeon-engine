# Aeon Time

Zero-dependency monotonic simulation clock, fixed-timestep accumulator (`Time`), and windowed frame-rate cap pacing (`FramePacer`).

## Scope
- **In scope**: Frame delta clamping, deterministic tick stepping, fixed-step accumulators, and FPS cap sleep budgets.
- **Out of scope**: Windowing (`winit`), OS threads, and ECS world state.
