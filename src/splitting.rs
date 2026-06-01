//! Splitting fields - minimal capability spaces that resolve all agent conflicts.
//!
//! The splitting field of a polynomial is the smallest field extension in which
//! the polynomial splits into linear factors. For agents: the minimal capability
//! extension that resolves all capability conflicts/dependencies.

use serde::{Deserialize, Serialize};
use crate::field::{Capability, CapabilityField};
use crate::extension::CapabilityExtension;

/// A "polynomial" over capabilities - represents a capability conflict or requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPolynomial {
    pub name: String,
    /// Roots: the capabilities needed to "resolve" this polynomial.
    pub roots: Vec<String>,
    /// Degree of the polynomial.
    pub degree: u32,
}

impl CapabilityPolynomial {
    /// Create a new capability polynomial.
    pub fn new(name: &str, roots: Vec<String>, degree: u32) -> Self {
        Self { name: name.to_string(), roots, degree }
    }

    /// A linear polynomial (degree 1) - already resolved.
    pub fn linear(name: &str, root: &str) -> Self {
        Self { name: name.to_string(), roots: vec![root.to_string()], degree: 1 }
    }

    /// A quadratic polynomial (degree 2) - needs two capabilities.
    pub fn quadratic(name: &str, root1: &str, root2: &str) -> Self {
        Self { name: name.to_string(), roots: vec![root1.to_string(), root2.to_string()], degree: 2 }
    }

    /// A quintic polynomial (degree 5) - the famous insolvable case.
    pub fn quintic(name: &str, roots: Vec<String>) -> Self {
        assert_eq!(roots.len(), 5);
        Self { name: name.to_string(), roots, degree: 5 }
    }

    /// Check if this polynomial is split over a field.
    pub fn is_split_over(&self, field: &CapabilityField) -> bool {
        self.roots.iter().all(|r| field.contains(r))
    }

    /// The roots that are NOT in the field.
    pub fn missing_roots(&self, field: &CapabilityField) -> Vec<&str> {
        self.roots.iter().filter(|r| !field.contains(r)).map(|s| s.as_str()).collect()
    }
}

/// Compute the splitting field of a set of capability polynomials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplittingField {
    pub base: CapabilityField,
    pub polynomials: Vec<CapabilityPolynomial>,
    pub splitting_extension: CapabilityExtension,
    pub all_roots_added: Vec<String>,
}

impl SplittingField {
    /// Compute the splitting field for a collection of polynomials over a base field.
    pub fn compute(base: CapabilityField, polynomials: Vec<CapabilityPolynomial>) -> Self {
        let mut all_roots = Vec::new();
        for poly in &polynomials {
            for root in &poly.roots {
                if !base.contains(root) && !all_roots.contains(root) {
                    all_roots.push(root.clone());
                }
            }
        }

        let new_caps: Vec<Capability> = all_roots.iter()
            .map(|r| Capability::new(r, 1))
            .collect();

        let extension = CapabilityExtension::new("splitting", base.clone(), new_caps);

        SplittingField {
            base,
            polynomials,
            splitting_extension: extension,
            all_roots_added: all_roots,
        }
    }

    /// Check if all polynomials split in the splitting field.
    pub fn verify_splitting(&self) -> bool {
        self.polynomials.iter().all(|poly| {
            poly.is_split_over(&self.splitting_extension.extended)
        })
    }

    /// The degree of the splitting field extension.
    pub fn degree(&self) -> usize {
        self.splitting_extension.degree()
    }

    /// Check if the splitting field is normal (it always should be, by construction).
    pub fn is_normal(&self) -> bool {
        self.splitting_extension.is_normal().is_normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_creation() {
        let p = CapabilityPolynomial::quadratic("pq", "a", "b");
        assert_eq!(p.degree, 2);
        assert_eq!(p.roots.len(), 2);
    }

    #[test]
    fn test_linear_polynomial() {
        let p = CapabilityPolynomial::linear("p", "a");
        assert_eq!(p.degree, 1);
        let mut f = CapabilityField::new("f");
        f.add_capability(Capability::new("a", 1));
        assert!(p.is_split_over(&f));
    }

    #[test]
    fn test_quadratic_not_split() {
        let p = CapabilityPolynomial::quadratic("p", "a", "b");
        let mut f = CapabilityField::new("f");
        f.add_capability(Capability::new("a", 1));
        assert!(!p.is_split_over(&f));
        assert_eq!(p.missing_roots(&f), vec!["b"]);
    }

    #[test]
    fn test_quintic_polynomial() {
        let p = CapabilityPolynomial::quintic("q", vec![
            "r1".into(), "r2".into(), "r3".into(), "r4".into(), "r5".into()
        ]);
        assert_eq!(p.degree, 5);
        assert_eq!(p.roots.len(), 5);
    }

    #[test]
    fn test_splitting_field_compute() {
        let base = CapabilityField::new("base");
        let polys = vec![
            CapabilityPolynomial::quadratic("p1", "a", "b"),
            CapabilityPolynomial::quadratic("p2", "c", "d"),
        ];
        let sf = SplittingField::compute(base, polys);
        assert_eq!(sf.all_roots_added.len(), 4);
    }

    #[test]
    fn test_splitting_field_verify() {
        let base = CapabilityField::new("base");
        let polys = vec![
            CapabilityPolynomial::quadratic("p1", "a", "b"),
        ];
        let sf = SplittingField::compute(base, polys);
        assert!(sf.verify_splitting());
    }

    #[test]
    fn test_splitting_field_degree() {
        let base = CapabilityField::new("base");
        let polys = vec![
            CapabilityPolynomial::quadratic("p1", "a", "b"),
            CapabilityPolynomial::linear("p2", "c"),
        ];
        let sf = SplittingField::compute(base, polys);
        assert_eq!(sf.degree(), 3);
    }

    #[test]
    fn test_missing_roots() {
        let p = CapabilityPolynomial::quadratic("p", "x", "y");
        let f = CapabilityField::new("empty");
        assert_eq!(p.missing_roots(&f), vec!["x", "y"]);
    }

    #[test]
    fn test_splitting_field_is_normal() {
        let base = CapabilityField::new("base");
        let polys = vec![
            CapabilityPolynomial::quadratic("p", "a", "b"),
        ];
        let sf = SplittingField::compute(base, polys);
        assert!(sf.is_normal());
    }
}
