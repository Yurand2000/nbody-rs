use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct QuadTreeCell {
    pub mass: f32,
    pub position: Vec2,
}

pub struct QuadTree<T> {
    pub cells: Vec<T>,
    pub depth: usize,
}

pub struct BodiesInfo {
    offset: Vec2,
    size: Vec2,
    min_distance: f32,
}

impl Simulator {
    pub fn get_quadtree(&self) -> (BodiesInfo, QuadTree<QuadTreeCell>) {
        // Compute bodies params
        let bodies_params = BodiesInfo::new(&self.bodies);

        // Compute quadtree
        let tree = self.build_quadtree(&bodies_params);

        (bodies_params, tree)
    }

    pub fn get_forces(&self, tree: &QuadTree<QuadTreeCell>, extra: &BodiesInfo) -> impl Iterator<Item = impl Iterator<Item = (Vec2, Vec2)>> {
        self.bodies.iter()
            .map(|body| Self::kick_force_iter(body, &self.params, tree, extra))
    }

    // Kick-Drift-Kick Leapfrog integration
    // with Quadtree approximation
    pub fn step_uniproc(&mut self) {
        // Compute bodies params
        let info = BodiesInfo::new(&self.bodies);

        // Compute quadtree
        let tree = self.build_quadtree(&info);

        // First kick
        self.kick(&tree, &info);

        // Drift
        self.drift();

        // Re-compute bodies params
        let info = BodiesInfo::new(&self.bodies);

        // Re-compute quadtree
        let tree = self.build_quadtree(&info);

        // Second kick
        self.kick(&tree, &info);
    }

    fn build_quadtree(&self, extra: &BodiesInfo) -> QuadTree<QuadTreeCell> {
        let mut tree: QuadTree<QuadTreeCell> = QuadTree::new(extra.max_depth());
        let max_depth = tree.depth;
        for depth in 0 ..= max_depth {
            let cells = tree.layer_at_depth_mut(depth);

            // Update cells
            for body in self.bodies.iter() {
                let normalized_pos = extra.normalize(body.position);
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
        extra: &BodiesInfo
    ) -> impl Iterator<Item = (Vec2, Vec2)> {
        tree.get_interactions(extra, body.position, params.theta / extra.size.magnitude())
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
                if diff.magnitude().powi(3) <= body.mass + cell_mass {
                    return None;
                }

                let direction = diff.normalize();
                let magnitude = (params.grav_constant * body.mass * cell_mass) /
                                    diff.magnitude_squared();

                Some((cell_position, direction * magnitude))
            })
    }

    fn kick(&mut self, tree: &QuadTree<QuadTreeCell>, extra: &BodiesInfo) {
        for body in self.bodies.iter_mut() {
            let force = Self::kick_force_iter(body, &self.params, tree, extra)
                .map(|(_, f)| f).sum::<Vec2>();

            let acceleration =
                if force.magnitude() < f32::EPSILON {
                    Vec2::zeros()
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

impl<T> QuadTree<T> {
    pub fn new(mut depth: usize) -> Self
        where T: Default + Clone
    {
        // max depth
        if depth > QuadTreeUtils::MAX_DEPTH_CPU {
            depth = QuadTreeUtils::MAX_DEPTH_CPU;
        }

        Self {
            cells: vec![T::default(); QuadTreeUtils::tree_size(depth)],
            depth,
        }
    }

    pub fn layer_at_depth(&self, depth: usize) -> &[T] {
        let start = QuadTreeUtils::layer_start(depth);
        let end = QuadTreeUtils::layer_end(depth);

        &self.cells[start..end]
    }

    pub fn layer_at_depth_mut(&mut self, depth: usize) -> &mut [T] {
        let start = QuadTreeUtils::layer_start(depth);
        let end = QuadTreeUtils::layer_end(depth);

        &mut self.cells[start..end]
    }
}

impl QuadTree<QuadTreeCell> {
    pub fn get_interactions(&self, info: &BodiesInfo, position: Vec2, theta: f32) -> impl Iterator<Item = (&QuadTreeCell, bool)> {
        QuadTreeIterator::new(self, info, position, theta)
    }
}

struct QuadTreeIteratorVisitData<'a> {
    curr_depth: usize,
    curr_cell_idx: &'a mut usize,
    max_cell_idx: usize,
}

struct QuadTreeIteratorVisitStack {
    visit_stack: Vec<(usize, usize)>,
}

impl QuadTreeIteratorVisitStack {
    fn new() -> Self {
        let mut visit_stack = Vec::with_capacity(QuadTreeUtils::MAX_DEPTH_CPU);
        visit_stack.push((0, 1));

        Self { visit_stack }
    }

    #[inline(always)]
    fn top_mut<'a>(&'a mut self) -> Option<QuadTreeIteratorVisitData<'a>> {
        let stack_size = self.visit_stack.len();

        self.visit_stack.last_mut()
            .map(|(curr_cell_idx, max_cell_idx)| QuadTreeIteratorVisitData {
                curr_depth: stack_size - 1,
                curr_cell_idx,
                max_cell_idx: *max_cell_idx
            })
    }

    #[inline(always)]
    fn pop(&mut self) {
        self.visit_stack.pop();
    }

    #[inline(always)]
    fn push(&mut self, position: usize) {
        self.visit_stack.push((position, position + 4));
    }
}

struct QuadTreeIterator<'a, T> {
    position: Vec2,
    theta: f32,
    quad_tree: &'a QuadTree<T>,
    visit_stack: QuadTreeIteratorVisitStack,
    nearest_cells: Vec<usize>,
}

impl<'a, T> QuadTreeIterator<'a, T> {
    pub fn new(quad_tree: &'a QuadTree<T>, info: &BodiesInfo, position: Vec2, theta: f32) -> Self {
        Self {
            position,
            theta,
            quad_tree,
            visit_stack: QuadTreeIteratorVisitStack::new(),
            nearest_cells: (0 ..= quad_tree.depth).map(|depth|
                QuadTreeUtils::nearest_cell_at_layer(depth, info.normalize(position))
            ).collect()
        }
    }

    /// Get cell of relative `index` at `depth`.
    #[inline(always)]
    fn cell_at_depth(&self, depth: usize, index: usize) -> &'a T {
        let offset = QuadTreeUtils::layer_start(depth);
        &self.quad_tree.cells[offset + index]
    }
}

impl<'a> Iterator for QuadTreeIterator<'a, QuadTreeCell> {
    type Item = (&'a QuadTreeCell, bool);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(QuadTreeIteratorVisitData {
            curr_depth,
            curr_cell_idx,
            max_cell_idx,
        }) = self.visit_stack.top_mut() {
            // go up one level if the whole subtree has been visited
            if *curr_cell_idx >= max_cell_idx {
                self.visit_stack.pop();
                continue;
            }

            // increment the current cell, since it has been visited
            // but keep the old index
            let curr_cell_idx =
                std::mem::replace(curr_cell_idx, *curr_cell_idx + 1);

            // stop the visit on a zero mass subtree
            let cell = self.cell_at_depth(curr_depth, curr_cell_idx);
            if cell.mass < f32::EPSILON {
                continue;
            }

            // visit cell
            let curr_width = QuadTreeUtils::layer_width(curr_depth);
            let ratio: f32 = curr_width / (self.position - cell.position).magnitude();
            if ratio < self.theta || curr_depth >= (self.quad_tree.depth - 1) {
                // return current cell
                let belongs_to_cell = curr_cell_idx == self.nearest_cells[curr_depth];
                return Some((cell, belongs_to_cell));
            } else {
                // visit subtree
                let curr_cell_subtree = QuadTreeUtils::first_position_at_next_layer(curr_cell_idx);
                self.visit_stack.push(curr_cell_subtree);
            }
        }

        None
    }
}

impl BodiesInfo {
    pub fn new(bodies: &[Body]) -> BodiesInfo {
        let mut min = Vec2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

        for Body { position, .. } in bodies {
            min.x = min.x.min(position.x);
            min.y = min.y.min(position.y);
            max.x = max.x.max(position.x);
            max.y = max.y.max(position.y);
        }

        let delta = 10.0;
        max += Vec2::new(delta, delta);
        min -= Vec2::new(delta, delta);

        BodiesInfo {
            offset: min,
            size: max - min,
            min_distance: 1.0,
        }
    }

    #[inline(always)]
    pub fn normalize(&self, vec: Vec2) -> Vec2 {
        let normalized = (vec - self.offset).component_div(&self.size);
        let normalized = Vec2::new(normalized.x, 1.0 - normalized.y);

        debug_assert!(
            normalized.x >= 0f32 && normalized.y >= 0f32 &&
            normalized.x < 1f32 && normalized.y < 1f32,
            "{:?} {:?} {:?} {:?}", vec, normalized, self.offset, self.size);

        normalized
    }

    #[inline(always)]
    pub fn invert_normalize(&self, vec: Vec2) -> Vec2 {
        let vec = Vec2::new(vec.x, 1.0 - vec.y);
        vec.component_mul(&self.size) + self.offset
    }

    #[inline(always)]
    pub fn invert_normalize_size(&self, vec: Vec2) -> Vec2 {
        vec.component_mul(&self.size)
    }

    #[inline(always)]
    pub fn max_depth(&self) -> usize {
        (self.size.x / self.min_distance).log2()
            .max((self.size.y / self.min_distance).log2())
            .max(1.0)
            .ceil() as usize
    }
}