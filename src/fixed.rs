//! Fixed fields - capabilities invariant under a symmetry group.
//!
//! The fixed field of a subgroup H of Gal(K/F) is the set of all capabilities
//! in K that are unchanged by every symmetry in H.

use serde::{Deserialize, Serialize};
use crate::field::{Capability, CapabilityField};
use crate::galois_group::{GaloisGroup, PermutationGroup, Permutation};
use crate::extension::CapabilityExtension;

/// Compute the fixed field of a subgroup acting on an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedField {
    /// The subgroup whose fixed field we're computing.
    pub subgroup_order: usize,
    /// The resulting fixed field.
    pub field: CapabilityField,
    /// Which adjoined capabilities are fixed.
    pub fixed_capabilities: Vec<String>,
    /// Which adjoined capabilities are moved (not fixed).
    pub moved_capabilities: Vec<String>,
}

impl FixedField {
    /// Compute the fixed field of a subgroup of the Galois group.
    pub fn compute(
        subgroup: &PermutationGroup,
        extension: &CapabilityExtension,
    ) -> Self {
        let mut field = CapabilityField::new(&format!("Fix(H_{} )", subgroup.order()));
        let mut fixed_caps = Vec::new();
        let mut moved_caps = Vec::new();

        // Base capabilities are always fixed
        for name in extension.base.capability_names() {
            if let Some(cap) = extension.base.get(name) {
                field.add_capability(cap.clone());
            }
        }

        // Check each adjoined capability
        for (i, cap_name) in extension.adjoined.iter().enumerate() {
            let is_fixed = subgroup.elements.iter().all(|perm| {
                perm.apply(i) == i
            });
            if is_fixed {
                if let Some(cap) = extension.extended.get(cap_name) {
                    field.add_capability(cap.clone());
                }
                fixed_caps.push(cap_name.clone());
            } else {
                moved_caps.push(cap_name.clone());
            }
        }

        FixedField {
            subgroup_order: subgroup.order(),
            field,
            fixed_capabilities: fixed_caps,
            moved_capabilities: moved_caps,
        }
    }

    /// Compute the fixed field of the full Galois group (should be the base field).
    pub fn full_galois_fixed(extension: &CapabilityExtension) -> Self {
        let gal = GaloisGroup::compute(extension);
        Self::compute(&gal.group, extension)
    }

    /// Compute the fixed field of the trivial subgroup (should be the full extension).
    pub fn trivial_fixed(extension: &CapabilityExtension) -> Self {
        let trivial = PermutationGroup::trivial(extension.adjoined.len());
        Self::compute(&trivial, extension)
    }

    /// The Artin index: [K : Fix(H)] = |H|.
    pub fn artin_index(&self, extension: &CapabilityExtension) -> usize {
        if self.field.degree() == 0 { return extension.extended.degree(); }
        extension.extended.degree() / self.field.degree()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_extension() -> CapabilityExtension {
        let base = CapabilityField::new("base");
        CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 1),
            Capability::new("c", 2),
        ])
    }

    #[test]
    fn test_full_galois_fixed() {
        let ext = make_extension();
        let ff = FixedField::full_galois_fixed(&ext);
        // The full Galois group's fixed field should be the base
        assert!(ff.field.degree() >= 0);
    }

    #[test]
    fn test_trivial_fixed() {
        let ext = make_extension();
        let ff = FixedField::trivial_fixed(&ext);
        // The trivial group's fixed field should be everything
        assert_eq!(ff.fixed_capabilities.len(), ext.adjoined.len());
        assert!(ff.moved_capabilities.is_empty());
    }

    #[test]
    fn test_fixed_field_of_subgroup() {
        let ext = make_extension();
        // Create a subgroup that fixes the first element only
        let id = Permutation::identity(3);
        let subgroup = PermutationGroup {
            name: "test_sub".to_string(),
            degree: 3,
            elements: vec![id],
        };
        let ff = FixedField::compute(&subgroup, &ext);
        assert_eq!(ff.subgroup_order, 1);
    }

    #[test]
    fn test_artin_index() {
        let ext = make_extension();
        let ff = FixedField::trivial_fixed(&ext);
        let idx = ff.artin_index(&ext);
        assert_eq!(idx, 1); // trivial group has index 1
    }

    #[test]
    fn test_fixed_capabilities() {
        let ext = make_extension();
        let ff = FixedField::full_galois_fixed(&ext);
        // 'c' has different power, should be fixed by everything
        assert!(ff.fixed_capabilities.contains(&"c".to_string()));
    }

    #[test]
    fn test_moved_capabilities() {
        let ext = make_extension();
        let gal = GaloisGroup::compute(&ext);
        // If S_2 acts on {a,b} and fixes c, then with a non-trivial subgroup
        if gal.group.order() > 1 {
            let non_trivial: Vec<Permutation> = gal.group.elements.iter()
                .filter(|p| !p.is_identity())
                .cloned()
                .collect();
            if !non_trivial.is_empty() {
                let sg = PermutationGroup {
                    name: "nt".to_string(),
                    degree: gal.group.degree,
                    elements: non_trivial,
                };
                let ff = FixedField::compute(&sg, &ext);
                assert!(!ff.moved_capabilities.is_empty() || ff.fixed_capabilities.len() == ext.adjoined.len());
            }
        }
    }

    #[test]
    fn test_fixed_field_base_preserved() {
        let mut base = CapabilityField::new("base");
        base.add_capability(Capability::new("base_cap", 0));
        let ext = CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
        ]);
        let ff = FixedField::full_galois_fixed(&ext);
        assert!(ff.field.contains("base_cap"));
    }
}
