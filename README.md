# lau-quipu

Inca quipu-inspired hierarchical tensor encoding. Position on the cord is the coordinate, knot type is the value, cord hierarchy is the dimension. This is a sparse tensor encoding that predates computer science by 500 years.

## The concept in 60 seconds

A **quipu** is a system of knotted cords used by the Inca to encode data. The main cord holds subsidiary cords. Each cord's position, knot type, and color encode different values. Hierarchy gives you dimensions — it's a sparse tensor before tensors had a name.

This crate implements quipu as a data structure:

- **Cords** are sequences of knots (values)
- **Knot types** encode different scales (units, tens, hundreds — or arbitrary values)
- **Hierarchical cords** give you multi-dimensional data
- **Encoding/decoding** converts between quipu representation and native Rust types
- **Tensor operations** on quipu-encoded data — add, multiply, contract

The insight: some data is naturally hierarchical. Quipu encode that hierarchy in the physical structure of the representation.

## Quick start

```rust
use lau_quipu::{Quipu, Cord, Knot, QuipuTensor};

// Build a quipu with hierarchical cords
let mut quipu = Quipu::new("census_data");

// Main cord with subsidiary cords
let mut population = Cord::new("population");
population.add_knot(Knot::figure_eight(100));  // 100
population.add_knot(Knot::long(30));           // 30
population.add_knot(Knot::single(5));          // 5

quipu.add_cord(population);

// Convert to tensor representation
let tensor = QuipuTensor::from_quipu(&quipu);
let total: f64 = tensor.sum();

// Encode/decode roundtrip
let encoded = quipu.encode();
let decoded = Quipu::decode(&encoded);
assert_eq!(quipu, decoded);
```

## Key types

| Type | What it is |
|------|-----------|
| `Quipu` | A collection of cords — the top-level data structure |
| `Cord` | A sequence of knots, optionally with subsidiary cords |
| `Knot` | A value encoding — figure-eight, long, single, or custom |
| `QuipuTensor` | Sparse tensor representation of quipu-encoded data |
| `QuipuEncoder` | Serialize/deserialize quipu to/from bytes |

## Contributing

[Open an issue](https://github.com/SuperInstance/lau-quipu/issues) or PR. Interesting directions:

- Color encoding as type metadata
- Multi-dimensional tensor operations on quipu
- Visualization of quipu structure
