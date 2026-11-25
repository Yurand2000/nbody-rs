use super::*;

pub struct UpSimulator {
    pub params: SimulationParams,
    bodies: Vec<Body>,
}

impl UpSimulator {
    pub fn new(params: SimulationParams) -> Self {
        Self { params, bodies: Vec::new() }
    }
}

impl Simulator for UpSimulator {
    fn get_params(&self) -> SimulationParams {
        self.params.clone()
    }

    fn set_params(&mut self, params: SimulationParams) {
        self.params = params;
    }

    fn reset(&mut self) {
        self.bodies.clear();
    }

    fn insert_body(&mut self, body: Body) -> BodyId {
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    // Kick-Drift-Kick Leapfrog integration
    // with Quadtree approximation
    fn step(&mut self) {
        // Compute quadtree normalization data
        let norm = NormalizationData::new(&self.bodies);

        // Compute quadtree
        let tree = self.build_quadtree(norm);

        // First kick
        self.kick(&tree);

        // Drift
        self.drift();

        // Re-compute quadtree normalization data
        let norm = NormalizationData::new(&self.bodies);

        // Re-compute quadtree
        let tree = self.build_quadtree(norm);

        // Second kick
        self.kick(&tree);
    }

    fn pull_updates(&mut self) { }

    fn get_bodies(&self) -> &[Body] {
        &self.bodies
    }

    fn get_body(&self, id: BodyId) -> &Body {
        &self.bodies[id]
    }
}

impl SimulatorExt for UpSimulator {
    fn get_quadtree(&self) -> Option<QuadTree<QuadTreeCell>> {
        // Compute quadtree normalization data
        let bodies_params = NormalizationData::new(&self.bodies);

        // Compute quadtree
        let tree = self.build_quadtree(bodies_params);

        Some(tree)
    }

    fn get_forces(&self) -> Option<Vec<Vec2f>> {
        let quadtree = self.get_quadtree()?;

        Some(
            self.bodies.iter()
                .map(|body|
                    Self::kick_force_iter(body, &self.params, &quadtree)
                        .map(|SingleBodyForce { force, .. }| force).sum::<Vec2f>()
                )
                .collect::<Vec<_>>()
        )
    }

    fn get_body_forces(&self, id: BodyId) -> Option<Vec<SingleBodyForce>> {
        let quadtree = self.get_quadtree()?;

        Some(Self::kick_force_iter(self.get_body(id), &self.params, &quadtree).collect())
    }
}

impl UpSimulator {
    fn build_quadtree(&self, norm: NormalizationData) -> QuadTree<QuadTreeCell> {
        let mut tree: QuadTree<QuadTreeCell> = QuadTree::new(norm.max_depth(), norm);
        let max_depth = tree.depth;
        for depth in 0 ..= max_depth {
            let cells = tree.layer_at_depth_mut(depth);

            // Update cells
            for body in self.bodies.iter() {
                let normalized_pos = norm.normalize(body.position);
                let idx = QuadTreeUtils::nearest_cell_at_layer(depth, normalized_pos);

                debug_assert!(idx < cells.len(), "{} {} {} {:?} {:?}", idx, cells.len(), depth, body.position, normalized_pos);

                cells[idx].mass += body.mass;
                cells[idx].position += body.position * body.mass;
            }

            // Average position from mass
            for cell in cells {
                if cell.mass > f32::EPSILON {
                    cell.position /= cell.mass;
                }
            }
        }

        tree
    }

    fn kick_force_iter(
        body: &Body,
        params: &SimulationParams,
        tree: &QuadTree<QuadTreeCell>,
    ) -> impl Iterator<Item = SingleBodyForce> {
        tree.get_interactions(body.position, params.theta)
            .flat_map(|(cell, belongs_to_cell)| {
                let (mut cell_mass, mut cell_position) = (cell.mass, cell.position);

                if belongs_to_cell {
                    cell_position *= cell_mass;
                    cell_mass -= body.mass;
                    if cell_mass < f32::EPSILON {
                        return None;
                    }

                    cell_position -= body.position * body.mass;
                    cell_position /= cell_mass;
                }

                let diff = cell_position - body.position;
                if diff.magnitude_squared() < f32::EPSILON {
                    return None;
                }

                if diff.magnitude().powi(3) <= body.mass + cell_mass {
                    return None;
                }

                let direction = diff.normalize();
                let magnitude = (params.g_constant * body.mass * cell_mass) /
                                    diff.magnitude_squared();

                Some(SingleBodyForce {
                    com: cell_position,
                    force: direction * magnitude
                })
            })
    }

    fn kick(&mut self, tree: &QuadTree<QuadTreeCell>) {
        for body in self.bodies.iter_mut() {
            let force = Self::kick_force_iter(body, &self.params, tree)
                .map(|SingleBodyForce { force, .. }| force).sum::<Vec2f>();

            let acceleration =
                if force.magnitude() < f32::EPSILON {
                    Vec2f::zeros()
                } else {
                    force / body.mass
                };

            // v <= v + \Delta_t / 2 * a
            body.velocity += self.params.time_step * acceleration / 2.0;
        }
    }

    fn drift(&mut self) {
        for body in self.bodies.iter_mut() {
            // x <= x + \Delta_t * v
            body.position += self.params.time_step * body.velocity;
        }
    }
}
