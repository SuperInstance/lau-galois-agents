# lau-galois-agents

Galois theory applied to agent capability spaces.

## Core Concept

Galois theory connects field extensions to groups. Applied to agents:
- The **"field"** is the agent's capability space
- The **"extension"** is adding new capabilities
- The **"Galois group"** is the symmetry group preserving agent behavior

## Features

- **Galois connection**: Two posets with adjoint functors (specialized from categorical-agents)
- **Fundamental theorem**: Subgroups ↔ intermediate fields ↔ intermediate capabilities
- **Normal extensions**: Capability extensions that preserve symmetry
- **Solvable groups**: Capabilities that can be built step-by-step
- **Insolvability of the quintic**: Some capability combinations CANNOT be constructed
- **Splitting field**: Minimal capability space resolving all agent conflicts
- **Fixed field**: Capabilities invariant under a symmetry group
- **Galois group computation**: Permutation group of capability roots
- **Constructibility analysis**: Determine which capabilities are constructible vs. impossible

## Usage

```rust
use lau_galois_agents::*;

// Create an agent with base capabilities
let mut agent = GaloisAgent::new("assistant", vec![
    ("perceive", 1),
    ("act", 1),
]);

// Extend with new capabilities
agent.extend("reasoning", vec![
    ("plan", 2),
    ("reason", 3),
]);

// Analyze the extension
let analysis = agent.analyze_extension(0).unwrap();
println!("{}", analysis.summary());
```

## Architecture

| Module | Description |
|--------|-------------|
| `poset` | Partially ordered sets (foundation) |
| `galois_connection` | Adjoint functors between posets |
| `field` | Capability fields (algebraic structure) |
| `extension` | Field extensions (adding capabilities) |
| `galois_group` | Symmetry groups and permutations |
| `fundamental_theorem` | Subgroup ↔ intermediate field bijection |
| `normal` | Normal extensions and conjugates |
| `solvable` | Solvable groups and step-by-step construction |
| `splitting` | Splitting fields for capability conflicts |
| `fixed` | Fixed fields under symmetry groups |
| `constructible` | Constructibility analysis |
| `agent` | High-level agent API |

## License

MIT
