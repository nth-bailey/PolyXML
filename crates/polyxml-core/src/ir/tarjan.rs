use std::cmp::min;
use std::collections::{HashMap, HashSet};

use super::{QName, SchemaIR, TypeDef, TypeRef};

/// Tarjan's Strongly Connected Components (SCC) analyzer for schema type graphs.
pub struct TarjanCycleDetector<'a> {
    ir: &'a SchemaIR,
    index: usize,
    indices: HashMap<QName, usize>,
    lowlinks: HashMap<QName, usize>,
    on_stack: HashSet<QName>,
    stack: Vec<QName>,
    sccs: Vec<Vec<QName>>,
}

impl<'a> TarjanCycleDetector<'a> {
    pub fn new(ir: &'a SchemaIR) -> Self {
        Self {
            ir,
            index: 0,
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            stack: Vec::new(),
            sccs: Vec::new(),
        }
    }

    /// Run Tarjan's SCC algorithm and return all strongly connected components
    /// with cycles (SCCs with size > 1, or size == 1 with a self-loop).
    pub fn find_cyclic_sccs(&mut self) -> Vec<Vec<QName>> {
        for qname in self.ir.types.keys() {
            if !self.indices.contains_key(qname) {
                self.strongconnect(qname);
            }
        }

        self.sccs
            .iter()
            .filter(|scc| {
                if scc.len() > 1 {
                    true
                } else if scc.len() == 1 {
                    // Check for self-loop
                    let node = &scc[0];
                    self.get_dependencies(node).contains(node)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    fn strongconnect(&mut self, node: &QName) {
        self.indices.insert(node.clone(), self.index);
        self.lowlinks.insert(node.clone(), self.index);
        self.index += 1;
        self.stack.push(node.clone());
        self.on_stack.insert(node.clone());

        let neighbors = self.get_dependencies(node);
        for neighbor in neighbors {
            if !self.indices.contains_key(&neighbor) {
                self.strongconnect(&neighbor);
                let neighbor_lowlink = *self.lowlinks.get(&neighbor).unwrap();
                let current_lowlink = self.lowlinks.get_mut(node).unwrap();
                *current_lowlink = min(*current_lowlink, neighbor_lowlink);
            } else if self.on_stack.contains(&neighbor) {
                let neighbor_index = *self.indices.get(&neighbor).unwrap();
                let current_lowlink = self.lowlinks.get_mut(node).unwrap();
                *current_lowlink = min(*current_lowlink, neighbor_index);
            }
        }

        if self.lowlinks.get(node) == self.indices.get(node) {
            let mut scc = Vec::new();
            while let Some(w) = self.stack.pop() {
                self.on_stack.remove(&w);
                scc.push(w.clone());
                if &w == node {
                    break;
                }
            }
            self.sccs.push(scc);
        }
    }

    /// Extract direct unboxed value dependencies for a given type.
    /// Excludes lists (as Vec<T>/vector<T> already introduce heap indirection)
    /// and already boxed references.
    fn get_dependencies(&self, qname: &QName) -> Vec<QName> {
        let mut deps = Vec::new();

        let Some(type_def) = self.ir.types.get(qname) else {
            return deps;
        };

        match type_def {
            TypeDef::Struct(s) => {
                for field in &s.fields {
                    if field.is_cycle_cut || field.cardinality.is_list() {
                        continue;
                    }
                    if let Some(target) = extract_named_type(&field.type_ref) {
                        if self.ir.types.contains_key(&target) {
                            deps.push(target);
                        }
                    }
                }
            }
            TypeDef::Union(u) => {
                for branch in &u.branches {
                    if let Some(target) = extract_named_type(&branch.type_ref) {
                        if self.ir.types.contains_key(&target) {
                            deps.push(target);
                        }
                    }
                }
            }
            TypeDef::Enum(_) | TypeDef::Simple(_) => {}
        }

        deps
    }
}

/// Helper to recursively extract a named type reference if it is not a List or Boxed.
fn extract_named_type(type_ref: &TypeRef) -> Option<QName> {
    match type_ref {
        TypeRef::Named(qname) => Some(qname.clone()),
        TypeRef::Primitive(_) | TypeRef::List(_) | TypeRef::Boxed(_) => None,
    }
}

/// Automatically detect circular dependencies and apply minimal cut points
/// to break infinite struct layouts.
pub fn resolve_cycles(ir: &mut SchemaIR) {
    loop {
        let cyclic_sccs = {
            let mut detector = TarjanCycleDetector::new(ir);
            detector.find_cyclic_sccs()
        };

        if cyclic_sccs.is_empty() {
            break;
        }

        let mut broke_any = false;

        for scc in cyclic_sccs {
            let scc_set: HashSet<QName> = scc.iter().cloned().collect();

            // Find best edge in this SCC to cut.
            // Preference:
            // 1. Optional field (min_occurs == 0) within the SCC
            // 2. Any field within the SCC
            let mut best_cut: Option<(QName, usize)> = None;
            let mut fallback_cut: Option<(QName, usize)> = None;

            for qname in &scc {
                if let Some(TypeDef::Struct(s)) = ir.types.get(qname) {
                    for (f_idx, field) in s.fields.iter().enumerate() {
                        if field.is_cycle_cut || field.cardinality.is_list() {
                            continue;
                        }
                        if let Some(target) = extract_named_type(&field.type_ref) {
                            if scc_set.contains(&target) {
                                if field.cardinality.is_optional() {
                                    best_cut = Some((qname.clone(), f_idx));
                                    break;
                                } else if fallback_cut.is_none() {
                                    fallback_cut = Some((qname.clone(), f_idx));
                                }
                            }
                        }
                    }
                }
                if best_cut.is_some() {
                    break;
                }
            }

            let cut_candidate = best_cut.or(fallback_cut);
            if let Some((type_qname, field_idx)) = cut_candidate {
                if let Some(TypeDef::Struct(s)) = ir.types.get_mut(&type_qname) {
                    let field = &mut s.fields[field_idx];
                    field.is_cycle_cut = true;
                    field.type_ref = TypeRef::Boxed(Box::new(field.type_ref.clone()));
                    broke_any = true;
                }
            }
        }

        if !broke_any {
            // Safety break to prevent infinite loop in case cycles cannot be cut
            break;
        }
    }
}
