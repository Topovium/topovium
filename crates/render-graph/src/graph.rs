// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashMap;

/// Identifies a pass within one graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PassId(pub u32);

/// Identifies a transient resource within one graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u32);

/// Why a graph could not be compiled.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    /// Passes form a cycle. Reported with the passes involved, because "graph has a
    /// cycle" without names is unactionable in a frame with forty passes.
    #[error("pass dependency cycle involving {0:?}")]
    Cycle(Vec<PassId>),
    /// A pass reads a resource that nothing writes.
    #[error("pass {pass:?} reads resource {resource:?}, which no pass writes")]
    UnwrittenRead { pass: PassId, resource: ResourceId },
    /// Two passes write the same resource, so the result depends on ordering.
    #[error("resource {resource:?} is written by both {first:?} and {second:?}")]
    DuplicateWrite {
        resource: ResourceId,
        first: PassId,
        second: PassId,
    },
}

#[derive(Debug, Clone)]
struct Pass {
    id: PassId,
    name: String,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
}

/// A frame under construction.
#[derive(Debug, Default)]
pub struct RenderGraph {
    passes: Vec<Pass>,
    next_pass: u32,
    next_resource: u32,
}

impl RenderGraph {
    /// An empty graph.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            passes: Vec::new(),
            next_pass: 0,
            next_resource: 0,
        }
    }

    /// Reserves an identifier for a transient resource.
    pub fn create_resource(&mut self) -> ResourceId {
        let id = ResourceId(self.next_resource);
        self.next_resource += 1;
        id
    }

    /// Adds a pass with the resources it reads and writes.
    pub fn add_pass(
        &mut self,
        name: impl Into<String>,
        reads: Vec<ResourceId>,
        writes: Vec<ResourceId>,
    ) -> PassId {
        let id = PassId(self.next_pass);
        self.next_pass += 1;
        self.passes.push(Pass {
            id,
            name: name.into(),
            reads,
            writes,
        });
        id
    }

    /// Number of passes.
    #[must_use]
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// A pass's name, for diagnostics and captures.
    #[must_use]
    pub fn pass_name(&self, id: PassId) -> Option<&str> {
        self.passes
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.as_str())
    }

    /// Orders passes so every producer runs before its consumers.
    ///
    /// # Errors
    /// Fails on a cycle, a read of a resource nothing writes, or two writers of one
    /// resource. All three are authoring mistakes that would otherwise show up as a
    /// backend-specific visual glitch on one vendor's driver.
    pub fn compile(&self) -> Result<Vec<PassId>, GraphError> {
        let mut producer: HashMap<ResourceId, PassId> = HashMap::new();
        for pass in &self.passes {
            for &resource in &pass.writes {
                if let Some(&first) = producer.get(&resource) {
                    return Err(GraphError::DuplicateWrite {
                        resource,
                        first,
                        second: pass.id,
                    });
                }
                producer.insert(resource, pass.id);
            }
        }

        let mut dependencies: HashMap<PassId, Vec<PassId>> = HashMap::new();
        let mut dependents: HashMap<PassId, Vec<PassId>> = HashMap::new();
        for pass in &self.passes {
            dependencies.entry(pass.id).or_default();
            for &resource in &pass.reads {
                let Some(&writer) = producer.get(&resource) else {
                    return Err(GraphError::UnwrittenRead {
                        pass: pass.id,
                        resource,
                    });
                };
                if writer != pass.id {
                    dependencies.entry(pass.id).or_default().push(writer);
                    dependents.entry(writer).or_default().push(pass.id);
                }
            }
        }

        // Kahn's algorithm, taking ready passes in declaration order so a valid graph
        // always compiles to the same schedule. A schedule that varies between runs
        // would make frame captures incomparable.
        let mut remaining: HashMap<PassId, usize> = dependencies
            .iter()
            .map(|(&id, deps)| (id, deps.len()))
            .collect();
        let mut ready: Vec<PassId> = self
            .passes
            .iter()
            .filter(|p| remaining.get(&p.id).copied().unwrap_or(0) == 0)
            .map(|p| p.id)
            .collect();

        let mut order = Vec::with_capacity(self.passes.len());
        while let Some(id) = ready.first().copied() {
            ready.remove(0);
            order.push(id);
            for &dependent in dependents.get(&id).into_iter().flatten() {
                if let Some(count) = remaining.get_mut(&dependent) {
                    *count -= 1;
                    if *count == 0 {
                        ready.push(dependent);
                    }
                }
            }
        }

        if order.len() != self.passes.len() {
            let stuck: Vec<PassId> = self
                .passes
                .iter()
                .map(|p| p.id)
                .filter(|id| !order.contains(id))
                .collect();
            return Err(GraphError::Cycle(stuck));
        }
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_are_ordered_by_their_data_dependencies() {
        let mut graph = RenderGraph::new();
        let depth = graph.create_resource();
        let hzb = graph.create_resource();
        let colour = graph.create_resource();

        // Declared out of order on purpose: the graph must sort them, not the author.
        let shade = graph.add_pass("shade", vec![hzb], vec![colour]);
        let prepass = graph.add_pass("depth-prepass", vec![], vec![depth]);
        let build_hzb = graph.add_pass("build-hzb", vec![depth], vec![hzb]);

        let order = graph.compile().unwrap();
        let position = |id: PassId| order.iter().position(|&p| p == id).unwrap();
        assert!(position(prepass) < position(build_hzb));
        assert!(position(build_hzb) < position(shade));
    }

    #[test]
    fn a_cycle_is_reported_with_the_passes_involved() {
        let mut graph = RenderGraph::new();
        let a = graph.create_resource();
        let b = graph.create_resource();
        graph.add_pass("first", vec![b], vec![a]);
        graph.add_pass("second", vec![a], vec![b]);

        match graph.compile() {
            Err(GraphError::Cycle(passes)) => assert_eq!(passes.len(), 2),
            other => panic!("expected a cycle, got {other:?}"),
        }
    }

    #[test]
    fn reading_a_resource_nobody_writes_is_an_error_not_a_black_screen() {
        let mut graph = RenderGraph::new();
        let orphan = graph.create_resource();
        graph.add_pass("consumer", vec![orphan], vec![]);
        assert!(matches!(
            graph.compile(),
            Err(GraphError::UnwrittenRead { .. })
        ));
    }

    #[test]
    fn two_writers_of_one_resource_are_rejected() {
        let mut graph = RenderGraph::new();
        let target = graph.create_resource();
        graph.add_pass("first", vec![], vec![target]);
        graph.add_pass("second", vec![], vec![target]);
        assert!(matches!(
            graph.compile(),
            Err(GraphError::DuplicateWrite { .. })
        ));
    }

    #[test]
    fn compilation_is_deterministic_across_runs() {
        let mut graph = RenderGraph::new();
        let a = graph.create_resource();
        graph.add_pass("write", vec![], vec![a]);
        graph.add_pass("read-one", vec![a], vec![]);
        graph.add_pass("read-two", vec![a], vec![]);

        let first = graph.compile().unwrap();
        for _ in 0..10 {
            assert_eq!(graph.compile().unwrap(), first);
        }
    }
}
