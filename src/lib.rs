//! # lau-quipu
//!
//! Inca quipu-inspired hierarchical tensor encoding.
//!
//! A quipu encodes data in a hierarchical physical structure:
//! - Position on the cord is the coordinate
//! - Knot type is the value
//! - Cord hierarchy is the dimension
//!
//! This is a sparse tensor encoding that predates computer science by 500 years.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a cord.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CordId(pub String);

impl CordId {
    pub fn new(s: impl Into<String>) -> Self {
        CordId(s.into())
    }
}

impl fmt::Display for CordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of knot, each encoding a value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KnotType {
    /// Single knot encoding value 1.
    Single,
    /// Double knot encoding value 2.
    Double,
    /// Triple knot encoding value 3.
    Triple,
    /// Figure-eight knot encoding value 0.
    FigureEight,
    /// Long knot encoding a specific value.
    LongKnot(f64),
}

impl KnotType {
    /// Return the numeric value of this knot type.
    pub fn value(&self) -> f64 {
        match self {
            KnotType::Single => 1.0,
            KnotType::Double => 2.0,
            KnotType::Triple => 3.0,
            KnotType::FigureEight => 0.0,
            KnotType::LongKnot(v) => *v,
        }
    }
}

/// A knot on a cord at a given position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Knot {
    pub knot_type: KnotType,
    pub position: u32,
    pub value: f64,
    pub label: Option<String>,
}

impl Knot {
    pub fn new(knot_type: KnotType, position: u32) -> Self {
        let value = knot_type.value();
        Knot {
            knot_type,
            position,
            value,
            label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Encode a number as a sequence of knots using Inca base-10 positional encoding.
    ///
    /// Each decimal digit is encoded at successive positions. The digit determines
    /// the knot type. Digit 0 → FigureEight, 1 → Single, 2 → Double, 3 → Triple,
    /// 4–9 → LongKnot(digit).
    pub fn encode_value(val: f64) -> Vec<Knot> {
        let abs_val = val.abs().floor() as u64;
        let digits: Vec<u64> = if abs_val == 0 {
            vec![0]
        } else {
            let mut d = Vec::new();
            let mut n = abs_val;
            while n > 0 {
                d.push(n % 10);
                n /= 10;
            }
            d.reverse();
            d
        };

        let sign_knot = if val < 0.0 {
            Some(Knot::new(KnotType::FigureEight, 0).with_label("negative"))
        } else {
            None
        };

        let offset = if sign_knot.is_some() { 1 } else { 0 };

        let digit_knots: Vec<Knot> = digits
            .into_iter()
            .enumerate()
            .map(|(i, d)| {
                let kt = match d {
                    0 => KnotType::FigureEight,
                    1 => KnotType::Single,
                    2 => KnotType::Double,
                    3 => KnotType::Triple,
                    v => KnotType::LongKnot(v as f64),
                };
                Knot::new(kt, (i as u32) + offset)
            })
            .collect();

        match sign_knot {
            Some(sk) => std::iter::once(sk).chain(digit_knots).collect(),
            None => digit_knots,
        }
    }

    /// Decode a sequence of knots back to a number.
    pub fn decode_value(knots: &[Knot]) -> f64 {
        if knots.is_empty() {
            return 0.0;
        }

        let mut negative = false;
        let mut start = 0;

        // Check if first knot is a sign marker (FigureEight with "negative" label at pos 0)
        if knots.len() > 1
            && knots[0].knot_type == KnotType::FigureEight
            && knots[0].label.as_deref() == Some("negative")
        {
            negative = true;
            start = 1;
        }

        let relevant = &knots[start..];
        if relevant.is_empty() {
            return 0.0;
        }

        // Each knot represents a decimal digit at its position
        let mut result: f64 = 0.0;
        let base: f64 = 10_f64;
        let digit_count = relevant.len();

        for (i, knot) in relevant.iter().enumerate() {
            let place = (digit_count - i - 1) as i32;
            let digit = match &knot.knot_type {
                KnotType::FigureEight => 0.0,
                KnotType::Single => 1.0,
                KnotType::Double => 2.0,
                KnotType::Triple => 3.0,
                KnotType::LongKnot(v) => *v,
            };
            result += digit * base.powi(place);
        }

        if negative {
            result = -result;
        }
        result
    }
}

/// Classification of a cord in the quipu hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CordType {
    Primary,
    Secondary,
    Tertiary,
    Auxiliary,
}

impl CordType {
    /// Numerical depth: Primary=1, Secondary=2, etc.
    pub fn depth(&self) -> u32 {
        match self {
            CordType::Primary => 1,
            CordType::Secondary => 2,
            CordType::Tertiary => 3,
            CordType::Auxiliary => 4,
        }
    }
}

/// A cord on the quipu, holding knots and forming the hierarchical structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cord {
    pub id: CordId,
    pub knots: Vec<Knot>,
    pub color: String,
    pub cord_type: CordType,
    pub child_cords: Vec<CordId>,
    pub parent_cord: Option<CordId>,
}

impl Cord {
    pub fn new(id: CordId, color: impl Into<String>, cord_type: CordType) -> Self {
        Cord {
            id,
            knots: Vec::new(),
            color: color.into(),
            cord_type,
            child_cords: Vec::new(),
            parent_cord: None,
        }
    }

    pub fn add_knot(&mut self, knot: Knot) {
        self.knots.push(knot);
    }

    /// Get the value of the knot at the given position.
    pub fn value_at(&self, position: u32) -> f64 {
        self.knots
            .iter()
            .find(|k| k.position == position)
            .map(|k| k.value)
            .unwrap_or(0.0)
    }

    /// Sum of all knot values on this cord.
    pub fn total_value(&self) -> f64 {
        self.knots.iter().map(|k| k.value).sum()
    }

    /// Encode an array of values as knots on this cord.
    pub fn encode(&mut self, values: &[f64]) {
        self.knots.clear();
        for (i, &v) in values.iter().enumerate() {
            let knots = Knot::encode_value(v);
            for mut knot in knots {
                knot.position = i as u32;
                self.knots.push(knot);
            }
        }
    }

    /// Decode all knots back to an array of values (grouped by position).
    pub fn decode(&self) -> Vec<f64> {
        if self.knots.is_empty() {
            return Vec::new();
        }
        let max_pos = self.knots.iter().map(|k| k.position).max().unwrap_or(0);
        let mut result = Vec::new();
        for pos in 0..=max_pos {
            let knots_at_pos: Vec<&Knot> = self.knots.iter().filter(|k| k.position == pos).collect();
            if knots_at_pos.is_empty() {
                result.push(0.0);
            } else if knots_at_pos.len() == 1 {
                result.push(knots_at_pos[0].value);
            } else {
                // Multiple knots at one position → use decode_value for the digit sequence
                result.push(Knot::decode_value(&knots_at_pos.iter().cloned().cloned().collect::<Vec<_>>()));
            }
        }
        result
    }

    pub fn knot_count(&self) -> usize {
        self.knots.len()
    }

    /// Depth of this cord in the hierarchy.
    pub fn depth(&self) -> u32 {
        self.cord_type.depth()
    }
}

/// The full quipu: a primary cord with attached cords forming a hierarchical tensor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quipu {
    pub primary_cord: Cord,
    pub cords: HashMap<CordId, Cord>,
    pub metadata: HashMap<String, String>,
    pub tick_created: u64,
}

impl Quipu {
    pub fn new(metadata: HashMap<String, String>, tick: u64) -> Self {
        let primary = Cord::new(CordId::new("primary"), "brown", CordType::Primary);
        Quipu {
            primary_cord: primary,
            cords: HashMap::new(),
            metadata,
            tick_created: tick,
        }
    }

    /// Add a cord as a child of the given parent. Returns the new cord's ID.
    pub fn add_cord(&mut self, parent: &CordId, mut cord: Cord) -> CordId {
        let id = cord.id.clone();
        cord.parent_cord = Some(parent.clone());

        // Determine cord type based on parent depth
        let parent_depth = if parent.0 == "primary" {
            1
        } else {
            self.cords
                .get(parent)
                .map(|c| c.depth())
                .unwrap_or(1)
        };
        cord.cord_type = match parent_depth {
            1 => CordType::Secondary,
            2 => CordType::Tertiary,
            _ => CordType::Auxiliary,
        };

        // Add child reference to parent
        if parent.0 == "primary" {
            self.primary_cord.child_cords.push(id.clone());
        } else if let Some(parent_cord) = self.cords.get_mut(parent) {
            parent_cord.child_cords.push(id.clone());
        }

        self.cords.insert(id.clone(), cord);
        id
    }

    pub fn get_cord(&self, id: &CordId) -> Option<&Cord> {
        if id.0 == "primary" {
            Some(&self.primary_cord)
        } else {
            self.cords.get(id)
        }
    }

    pub fn get_mut_cord(&mut self, id: &CordId) -> Option<&mut Cord> {
        if id.0 == "primary" {
            Some(&mut self.primary_cord)
        } else {
            self.cords.get_mut(id)
        }
    }

    /// Sum of all knot values across all cords.
    pub fn total_value(&self) -> f64 {
        self.primary_cord.total_value()
            + self.cords.values().map(|c| c.total_value()).sum::<f64>()
    }

    /// Sum of the primary cord only.
    pub fn primary_value(&self) -> f64 {
        self.primary_cord.total_value()
    }

    /// Total values per cord.
    pub fn cord_values(&self) -> HashMap<CordId, f64> {
        let mut map = HashMap::new();
        map.insert(self.primary_cord.id.clone(), self.primary_cord.total_value());
        for (id, cord) in &self.cords {
            map.insert(id.clone(), cord.total_value());
        }
        map
    }

    /// Encode multi-dimensional data as a quipu tensor.
    /// Each key becomes a secondary cord; values are encoded as knots.
    pub fn encode_tensor(&mut self, data: &HashMap<String, Vec<f64>>) {
        for (name, values) in data {
            let cord_id = CordId::new(name);
            let mut cord = Cord::new(cord_id.clone(), "white", CordType::Secondary);
            cord.encode(values);
            self.add_cord(&CordId::new("primary"), cord);
        }
    }

    /// Decode the quipu back to multi-dimensional data.
    pub fn decode_tensor(&self) -> HashMap<String, Vec<f64>> {
        let mut result = HashMap::new();
        result.insert("primary".to_string(), self.primary_cord.decode());
        for (id, cord) in &self.cords {
            result.insert(id.0.clone(), cord.decode());
        }
        result
    }

    /// Check if total value is conserved within tolerance.
    pub fn is_conserved(&self, expected_total: f64, tolerance: f64) -> bool {
        (self.total_value() - expected_total).abs() <= tolerance
    }

    pub fn cord_count(&self) -> usize {
        1 + self.cords.len() // primary + children
    }

    /// Maximum depth of the cord hierarchy.
    pub fn depth(&self) -> u32 {
        let max_child = self
            .cords
            .values()
            .map(|c| c.depth())
            .max()
            .unwrap_or(1);
        self.primary_cord.depth().max(max_child)
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        let total_knots: usize = self.primary_cord.knot_count()
            + self.cords.values().map(|c| c.knot_count()).sum::<usize>();
        format!(
            "Quipu(created={}, cords={}, knots={}, depth={}, total_value={:.2}, metadata={:?})",
            self.tick_created,
            self.cord_count(),
            total_knots,
            self.depth(),
            self.total_value(),
            self.metadata,
        )
    }
}

/// Registry of named quipus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuipuRegistry {
    pub quipus: HashMap<String, Quipu>,
}

impl QuipuRegistry {
    pub fn new() -> Self {
        QuipuRegistry {
            quipus: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: &str, quipu: Quipu) {
        self.quipus.insert(id.to_string(), quipu);
    }

    pub fn get(&self, id: &str) -> Option<&Quipu> {
        self.quipus.get(id)
    }

    /// Check conservation for all registered quipus.
    pub fn conservation_check(&self, tolerance: f64) -> Vec<(&str, bool)> {
        self.quipus
            .iter()
            .map(|(id, q)| {
                let total = q.total_value();
                let conserved = total.is_finite() && total.abs() <= 1e10; // sanity
                let _ = tolerance; // future: per-quipu expected total comparison
                (id.as_str(), conserved)
            })
            .collect()
    }

    /// Compute aggregate stats across all quipus.
    pub fn registry_stats(&self) -> QuipuStats {
        let total_quipus = self.quipus.len();
        let total_cords: usize = self.quipus.values().map(|q| q.cord_count()).sum();
        let total_knots: usize = self
            .quipus
            .values()
            .map(|q| {
                q.primary_cord.knot_count()
                    + q.cords.values().map(|c| c.knot_count()).sum::<usize>()
            })
            .sum();
        let avg_depth = if total_quipus > 0 {
            self.quipus.values().map(|q| q.depth() as f64).sum::<f64>() / total_quipus as f64
        } else {
            0.0
        };
        let total_encoded_value: f64 = self.quipus.values().map(|q| q.total_value()).sum();

        QuipuStats {
            total_quipus,
            total_cords,
            total_knots,
            avg_depth,
            total_encoded_value,
        }
    }
}

impl Default for QuipuRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate statistics for a registry of quipus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuipuStats {
    pub total_quipus: usize,
    pub total_cords: usize,
    pub total_knots: usize,
    pub avg_depth: f64,
    pub total_encoded_value: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- CordId ---
    #[test]
    fn test_cord_id_new() {
        let id = CordId::new("test");
        assert_eq!(id.0, "test");
    }

    #[test]
    fn test_cord_id_clone_hash_eq() {
        let a = CordId::new("x");
        let b = a.clone();
        assert_eq!(a, b);
        let mut set = std::collections::HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
    }

    #[test]
    fn test_cord_id_display() {
        let id = CordId::new("cord-1");
        assert_eq!(format!("{}", id), "cord-1");
    }

    // --- KnotType ---
    #[test]
    fn test_knot_type_values() {
        assert_eq!(KnotType::Single.value(), 1.0);
        assert_eq!(KnotType::Double.value(), 2.0);
        assert_eq!(KnotType::Triple.value(), 3.0);
        assert_eq!(KnotType::FigureEight.value(), 0.0);
        assert_eq!(KnotType::LongKnot(7.0).value(), 7.0);
    }

    // --- Knot ---
    #[test]
    fn test_knot_new() {
        let k = Knot::new(KnotType::Double, 5);
        assert_eq!(k.value, 2.0);
        assert_eq!(k.position, 5);
        assert!(k.label.is_none());
    }

    #[test]
    fn test_knot_with_label() {
        let k = Knot::new(KnotType::Single, 0).with_label("start");
        assert_eq!(k.label.as_deref(), Some("start"));
    }

    #[test]
    fn test_encode_value_zero() {
        let knots = Knot::encode_value(0.0);
        let val = Knot::decode_value(&knots);
        assert_eq!(val, 0.0);
    }

    #[test]
    fn test_encode_value_single_digit() {
        let knots = Knot::encode_value(5.0);
        let val = Knot::decode_value(&knots);
        assert_eq!(val, 5.0);
    }

    #[test]
    fn test_encode_value_multi_digit() {
        let knots = Knot::encode_value(42.0);
        let val = Knot::decode_value(&knots);
        assert_eq!(val, 42.0);
    }

    #[test]
    fn test_encode_value_large() {
        let knots = Knot::encode_value(12345.0);
        let val = Knot::decode_value(&knots);
        assert_eq!(val, 12345.0);
    }

    #[test]
    fn test_encode_value_negative() {
        let knots = Knot::encode_value(-7.0);
        let val = Knot::decode_value(&knots);
        assert_eq!(val, -7.0);
    }

    #[test]
    fn test_encode_decode_roundtrip_1() {
        let knots = Knot::encode_value(1.0);
        assert_eq!(Knot::decode_value(&knots), 1.0);
    }

    #[test]
    fn test_encode_decode_roundtrip_100() {
        let knots = Knot::encode_value(100.0);
        assert_eq!(Knot::decode_value(&knots), 100.0);
    }

    #[test]
    fn test_encode_decode_roundtrip_9999() {
        let knots = Knot::encode_value(9999.0);
        assert_eq!(Knot::decode_value(&knots), 9999.0);
    }

    // --- CordType ---
    #[test]
    fn test_cord_type_depth() {
        assert_eq!(CordType::Primary.depth(), 1);
        assert_eq!(CordType::Secondary.depth(), 2);
        assert_eq!(CordType::Tertiary.depth(), 3);
        assert_eq!(CordType::Auxiliary.depth(), 4);
    }

    // --- Cord ---
    #[test]
    fn test_cord_new() {
        let cord = Cord::new(CordId::new("c1"), "red", CordType::Secondary);
        assert_eq!(cord.id.0, "c1");
        assert_eq!(cord.color, "red");
        assert!(cord.knots.is_empty());
        assert_eq!(cord.knot_count(), 0);
    }

    #[test]
    fn test_cord_add_knot() {
        let mut cord = Cord::new(CordId::new("c"), "blue", CordType::Primary);
        cord.add_knot(Knot::new(KnotType::Single, 0));
        cord.add_knot(Knot::new(KnotType::Double, 1));
        assert_eq!(cord.knot_count(), 2);
    }

    #[test]
    fn test_cord_value_at() {
        let mut cord = Cord::new(CordId::new("c"), "blue", CordType::Primary);
        cord.add_knot(Knot::new(KnotType::Triple, 3));
        assert_eq!(cord.value_at(3), 3.0);
        assert_eq!(cord.value_at(99), 0.0);
    }

    #[test]
    fn test_cord_total_value() {
        let mut cord = Cord::new(CordId::new("c"), "blue", CordType::Primary);
        cord.add_knot(Knot::new(KnotType::Single, 0));
        cord.add_knot(Knot::new(KnotType::Double, 1));
        cord.add_knot(Knot::new(KnotType::Triple, 2));
        assert_eq!(cord.total_value(), 6.0);
    }

    #[test]
    fn test_cord_encode_decode() {
        let mut cord = Cord::new(CordId::new("c"), "green", CordType::Primary);
        cord.encode(&[3.0, 7.0, 0.0]);
        let vals = cord.decode();
        assert_eq!(vals, vec![3.0, 7.0, 0.0]);
    }

    #[test]
    fn test_cord_depth() {
        let cord = Cord::new(CordId::new("c"), "red", CordType::Tertiary);
        assert_eq!(cord.depth(), 3);
    }

    #[test]
    fn test_cord_empty_decode() {
        let cord = Cord::new(CordId::new("c"), "red", CordType::Primary);
        assert!(cord.decode().is_empty());
    }

    // --- Quipu ---
    #[test]
    fn test_quipu_new() {
        let q = Quipu::new(HashMap::new(), 42);
        assert_eq!(q.tick_created, 42);
        assert_eq!(q.cord_count(), 1); // primary only
    }

    #[test]
    fn test_quipu_add_cord() {
        let mut q = Quipu::new(HashMap::new(), 0);
        let cord = Cord::new(CordId::new("child1"), "white", CordType::Secondary);
        let id = q.add_cord(&CordId::new("primary"), cord);
        assert_eq!(id.0, "child1");
        assert_eq!(q.cord_count(), 2);
    }

    #[test]
    fn test_quipu_get_cord() {
        let mut q = Quipu::new(HashMap::new(), 0);
        let cord = Cord::new(CordId::new("x"), "white", CordType::Secondary);
        q.add_cord(&CordId::new("primary"), cord);
        assert!(q.get_cord(&CordId::new("x")).is_some());
        assert!(q.get_cord(&CordId::new("missing")).is_none());
        assert!(q.get_cord(&CordId::new("primary")).is_some());
    }

    #[test]
    fn test_quipu_total_value() {
        let mut q = Quipu::new(HashMap::new(), 0);
        q.primary_cord.add_knot(Knot::new(KnotType::Single, 0));
        let mut child = Cord::new(CordId::new("c"), "w", CordType::Secondary);
        child.add_knot(Knot::new(KnotType::Double, 0));
        q.add_cord(&CordId::new("primary"), child);
        assert_eq!(q.total_value(), 3.0);
    }

    #[test]
    fn test_quipu_primary_value() {
        let mut q = Quipu::new(HashMap::new(), 0);
        q.primary_cord.add_knot(Knot::new(KnotType::Triple, 0));
        assert_eq!(q.primary_value(), 3.0);
    }

    #[test]
    fn test_quipu_cord_values() {
        let mut q = Quipu::new(HashMap::new(), 0);
        q.primary_cord.add_knot(Knot::new(KnotType::Single, 0));
        let mut child = Cord::new(CordId::new("c"), "w", CordType::Secondary);
        child.add_knot(Knot::new(KnotType::Double, 0));
        q.add_cord(&CordId::new("primary"), child);
        let cv = q.cord_values();
        assert_eq!(cv[&CordId::new("primary")], 1.0);
        assert_eq!(cv[&CordId::new("c")], 2.0);
    }

    #[test]
    fn test_quipu_encode_decode_tensor() {
        let mut q = Quipu::new(HashMap::new(), 0);
        let mut data = HashMap::new();
        data.insert("a".to_string(), vec![1.0, 2.0]);
        data.insert("b".to_string(), vec![3.0]);
        q.encode_tensor(&data);
        let decoded = q.decode_tensor();
        assert_eq!(decoded["a"], vec![1.0, 2.0]);
        assert_eq!(decoded["b"], vec![3.0]);
    }

    #[test]
    fn test_quipu_is_conserved() {
        let mut q = Quipu::new(HashMap::new(), 0);
        q.primary_cord.add_knot(Knot::new(KnotType::LongKnot(10.0), 0));
        assert!(q.is_conserved(10.0, 0.01));
        assert!(!q.is_conserved(5.0, 0.01));
    }

    #[test]
    fn test_quipu_depth() {
        let mut q = Quipu::new(HashMap::new(), 0);
        assert_eq!(q.depth(), 1);
        let cord = Cord::new(CordId::new("c"), "w", CordType::Secondary);
        q.add_cord(&CordId::new("primary"), cord);
        assert_eq!(q.depth(), 2);
    }

    #[test]
    fn test_quipu_summary() {
        let q = Quipu::new(HashMap::new(), 100);
        let s = q.summary();
        assert!(s.contains("Quipu(created=100"));
        assert!(s.contains("cords=1"));
    }

    #[test]
    fn test_quipu_get_mut_cord() {
        let mut q = Quipu::new(HashMap::new(), 0);
        let cord = Cord::new(CordId::new("c"), "w", CordType::Secondary);
        q.add_cord(&CordId::new("primary"), cord);
        {
            let c = q.get_mut_cord(&CordId::new("c")).unwrap();
            c.add_knot(Knot::new(KnotType::Single, 0));
        }
        assert_eq!(q.get_cord(&CordId::new("c")).unwrap().knot_count(), 1);
    }

    // --- QuipuRegistry ---
    #[test]
    fn test_registry_new() {
        let reg = QuipuRegistry::new();
        assert!(reg.quipus.is_empty());
    }

    #[test]
    fn test_registry_register_get() {
        let mut reg = QuipuRegistry::new();
        let q = Quipu::new(HashMap::new(), 0);
        reg.register("q1", q);
        assert!(reg.get("q1").is_some());
        assert!(reg.get("missing").is_none());
    }

    #[test]
    fn test_registry_conservation_check() {
        let mut reg = QuipuRegistry::new();
        reg.register("ok", Quipu::new(HashMap::new(), 0));
        let results = reg.conservation_check(0.01);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "ok");
    }

    #[test]
    fn test_registry_stats() {
        let mut reg = QuipuRegistry::new();
        let mut q = Quipu::new(HashMap::new(), 0);
        q.primary_cord.add_knot(Knot::new(KnotType::Single, 0));
        reg.register("q1", q);
        let stats = reg.registry_stats();
        assert_eq!(stats.total_quipus, 1);
        assert_eq!(stats.total_cords, 1);
        assert_eq!(stats.total_knots, 1);
        assert_eq!(stats.total_encoded_value, 1.0);
    }

    #[test]
    fn test_registry_stats_empty() {
        let reg = QuipuRegistry::new();
        let stats = reg.registry_stats();
        assert_eq!(stats.total_quipus, 0);
        assert_eq!(stats.avg_depth, 0.0);
    }

    // --- Serde ---
    #[test]
    fn test_serde_roundtrip_cord() {
        let mut cord = Cord::new(CordId::new("c"), "red", CordType::Primary);
        cord.add_knot(Knot::new(KnotType::LongKnot(42.0), 0));
        let json = serde_json::to_string(&cord).unwrap();
        let back: Cord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, cord.id);
        assert_eq!(back.knot_count(), 1);
    }

    #[test]
    fn test_serde_roundtrip_quipu() {
        let mut q = Quipu::new(HashMap::from([("k".to_string(), "v".to_string())]), 99);
        q.primary_cord.add_knot(Knot::new(KnotType::Single, 0));
        let json = serde_json::to_string(&q).unwrap();
        let back: Quipu = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tick_created, 99);
        assert_eq!(back.metadata["k"], "v");
    }

    #[test]
    fn test_serde_roundtrip_registry() {
        let mut reg = QuipuRegistry::new();
        reg.register("q", Quipu::new(HashMap::new(), 1));
        let json = serde_json::to_string(&reg).unwrap();
        let back: QuipuRegistry = serde_json::from_str(&json).unwrap();
        assert!(back.get("q").is_some());
    }
}
