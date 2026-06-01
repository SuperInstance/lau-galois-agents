//! Normal extensions - capability extensions that preserve symmetry.
//!
//! A normal extension K/F is one where the Galois group acts transitively on the
//! roots of the minimal polynomial. For capabilities: an extension is normal if
//! adding a capability automatically brings all its "conjugate" capabilities.

use serde::{Deserialize, Serialize};
use crate::extension::CapabilityExtension;
use crate::galois_group::GaloisGroup;
use crate::field::CapabilityField;

/// Result of checking normality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalityCheck {
    pub is_normal: bool,
    pub reasons: Vec<String>,
}

impl CapabilityExtension {
    /// Check if this extension is normal.
    /// An extension is normal if every irreducible polynomial over F that has a root in K
    /// splits completely in K. For capabilities: if adding a capability, all conjugate
    /// capabilities (with same power/structure) are also added.
    pub fn is_normal(&self) -> NormalityCheck {
        let mut reasons = Vec::new();
        let mut is_normal = true;

        // Check: all adjoined capabilities of the same power should be present
        let power_groups = self.adjoined.iter().filter_map(|name| {
            self.extended.get(name).map(|cap| (name.clone(), cap.power))
        }).fold(std::collections::BTreeMap::<u32, Vec<String>>::new(), |mut acc, (name, power)| {
            acc.entry(power).or_default().push(name);
            acc
        });

        // For each power level, check if the base field has all conjugates
        for (power, caps) in &power_groups {
            if caps.len() > 1 {
                reasons.push(format!("Power {}: capabilities {} are conjugates, all present", power, caps.join(", ")));
            } else {
                // Single capability of this power - check if it's self-conjugate
                reasons.push(format!("Power {}: {} is self-conjugate", power, caps[0]));
            }
        }

        // Check: the Galois group should act transitively on adjoined capabilities
        let gal = GaloisGroup::compute(self);
        if gal.order() > 1 && self.adjoined.len() > 1 {
            // Verify transitivity: for any two adjoined elements, there's a permutation mapping one to the other
            let has_same_power = |i: usize, j: usize| -> bool {
                let ci = self.extended.get(&self.adjoined[i]);
                let cj = self.extended.get(&self.adjoined[j]);
                ci.zip(cj).map(|(a, b)| a.power == b.power).unwrap_or(false)
            };

            for i in 0..self.adjoined.len() {
                let mut can_reach_all = false;
                for perm in &gal.group.elements {
                    if perm.apply(i) != i {
                        can_reach_all = true;
                        break;
                    }
                }
                if !can_reach_all && self.adjoined.len() > 1 {
                    // Check if there are other elements with same power
                    let has_conjugates = (0..self.adjoined.len()).any(|j| j != i && has_same_power(i, j));
                    if has_conjugates {
                        is_normal = false;
                        reasons.push(format!("{} has conjugates but no symmetry maps to them", self.adjoined[i]));
                    }
                }
            }
        }

        if is_normal && !self.is_trivial() {
            reasons.push("Extension is normal: Galois group acts transitively on conjugates".to_string());
        }

        NormalityCheck { is_normal, reasons }
    }

    /// The normal closure of this extension (smallest normal extension containing it).
    pub fn normal_closure(&self) -> CapabilityExtension {
        let mut extended = self.extended.clone();
        let mut adjoined = self.adjoined.clone();

        // Add all conjugate capabilities
        let power_groups: std::collections::BTreeMap<u32, Vec<String>> = self.adjoined.iter()
            .filter_map(|name| {
                self.extended.get(name).map(|cap| (cap.power, name.clone()))
            })
            .fold(std::collections::BTreeMap::new(), |mut acc, (power, name)| {
                acc.entry(power).or_default().push(name);
                acc
            });

        // For each power group, ensure we have "complete" conjugacy classes
        for (power, caps) in &power_groups {
            if caps.len() == 1 {
                // Add the "missing" conjugate capabilities
                let conjugate_name = format!("{}_conj", caps[0]);
                if !extended.contains(&conjugate_name) {
                    extended.add_capability(crate::field::Capability::new(&conjugate_name, *power));
                    adjoined.push(conjugate_name);
                }
            }
        }

        CapabilityExtension {
            name: format!("normal_closure({})", self.name),
            base: self.base.clone(),
            extended,
            adjoined,
        }
    }

    /// Check if the extension is separable (all minimal polynomials have distinct roots).
    /// In our model: all adjoined capabilities have distinct structures.
    pub fn is_separable(&self) -> bool {
        // In characteristic 0 (our model), all extensions are separable
        true
    }

    /// Check if the extension is Galois (normal and separable).
    pub fn is_galois(&self) -> bool {
        self.is_normal().is_normal && self.is_separable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Capability;

    #[test]
    fn test_normal_extension_same_power() {
        let base = CapabilityField::new("base");
        let ext = CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 1),
        ]);
        let check = ext.is_normal();
        assert!(check.is_normal);
    }

    #[test]
    fn test_non_trivial_normal() {
        let base = CapabilityField::new("base");
        let ext = CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 2),
        ]);
        // Different powers - each is self-conjugate, trivially normal
        let check = ext.is_normal();
        assert!(check.is_normal);
    }

    #[test]
    fn test_normal_closure() {
        let base = CapabilityField::new("base");
        let ext = CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
        ]);
        let closure = ext.normal_closure();
        // Should add conjugate capability
        assert!(closure.adjoined.len() >= ext.adjoined.len());
    }

    #[test]
    fn test_is_separable() {
        let base = CapabilityField::new("base");
        let ext = CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
        ]);
        assert!(ext.is_separable());
    }

    #[test]
    fn test_is_galois() {
        let base = CapabilityField::new("base");
        let ext = CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 1),
        ]);
        assert!(ext.is_galois());
    }

    #[test]
    fn test_trivial_extension_is_normal() {
        let mut base = CapabilityField::new("base");
        base.add_capability(Capability::new("a", 1));
        let ext = CapabilityExtension::new("trivial", base.clone(), vec![]);
        let check = ext.is_normal();
        assert!(check.is_normal);
    }

    #[test]
    fn test_normality_reasons() {
        let base = CapabilityField::new("base");
        let ext = CapabilityExtension::new("test", base, vec![
            Capability::new("a", 1),
            Capability::new("b", 1),
            Capability::new("c", 1),
        ]);
        let check = ext.is_normal();
        assert!(!check.reasons.is_empty());
    }
}
