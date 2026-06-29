use super::*;

#[derive(Debug, Default, Clone, Copy)]
struct QuadTreeCell {
    mass: f32,
    position: Vec2,
}

struct QuadTree<T> {
    cells: Vec<T>,
    depth: usize,
}

struct ExtraParams {
    min: f32,
    max: f32,
    min_distance: f32,
}

impl Simulator {
    // Kick-Drift-Kick Leapfrog integration
    // with Quadtree approximation
    pub fn step_uniproc(&mut self) {
        // Compute bodies params
        let bodies_params = ExtraParams::from_bodies(&self.bodies);

        // Compute quadtree
        let tree = self.build_quadtree(&bodies_params);

        // First kick
        self.kick(&tree, &bodies_params);

        // Drift
        self.drift(&bodies_params);

        // Re-compute bodies params
        let bodies_params = ExtraParams::from_bodies(&self.bodies);

        // Re-compute quadtree
        let tree = self.build_quadtree(&bodies_params);

        // Second kick
        self.kick(&tree, &bodies_params);
    }

    fn build_quadtree(&self, extra: &ExtraParams) -> QuadTree<QuadTreeCell> {
        let mut tree: QuadTree<QuadTreeCell> = QuadTree::new(extra.max_depth());
        let max_depth = tree.depth;
        for depth in 0..max_depth {
            let cells = tree.layer_at_depth_mut(depth);

            // Update cells
            for body in self.bodies.iter() {
                let normalized_pos = extra.normalize(body.position);
                let idx = QuadTreeUtils::nearest_cell_at_layer(depth, normalized_pos);

                cells[idx].mass += body.mass;
                cells[idx].position += body.position * body.mass;
            }

            // Average position from mass
            for cell in cells {
                if cell.mass <= f32::EPSILON {
                    cell.position /= cell.mass;
                }
            }
        }

        tree
    }

    fn kick(&mut self, tree: &QuadTree<QuadTreeCell>, extra: &ExtraParams) {
        for body in self.bodies.iter_mut() {
            // compute total force
            let normalized_pos = extra.normalize(body.position);
            let mut force = Vec2::zeros();
            for (cell, belongs_to_cell) in tree.get_interactions(normalized_pos, self.params.theta) {
                let (mut cell_mass, mut cell_position) = (cell.mass, cell.position);
                if belongs_to_cell {
                    cell_position *= cell_mass;
                    cell_mass -= body.mass;
                    cell_position -= body.position * body.mass;
                    cell_position /= cell_mass;
                }

                if cell_mass <= f32::EPSILON {
                    continue;
                }

                let diff = cell_position - body.position;
                if diff.magnitude() <= f32::EPSILON {
                    continue;
                }

                // println!("{:?} {:?} {} {:?} {}", body.position, cell.position, cell.mass, cell_position, cell_mass);

                let direction = diff.normalize();
                let magnitude = (self.params.grav_constant * body.mass * cell_mass) /
                                    diff.magnitude();

                force += direction * magnitude;
            }

            let acceleration =
                if force.magnitude() < f32::EPSILON {
                    Vec2::zeros()
                } else {
                    force / body.mass
                };

            // println!("{} {}", force.x, force.y);

            // v <= v + \Delta_t / 2 * a
            body.velocity += self.params.time_step * acceleration / 2.0;
        }
    }

    fn drift(&mut self, _: &ExtraParams) {
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
        if depth > QuadTreeUtils::MAX_DEPTH {
            depth = QuadTreeUtils::MAX_DEPTH;
        }

        Self {
            cells: vec![T::default(); QuadTreeUtils::tree_size(depth)],
            depth,
        }
    }

    pub fn layer_at_depth_mut(&mut self, depth: usize) -> &mut [T] {
        let start = QuadTreeUtils::layer_start(depth);
        let end = QuadTreeUtils::layer_end(depth);

        &mut self.cells[start..end]
    }

    pub fn get_interactions(&self, position: Vec2, theta: f32) -> impl Iterator<Item = (&T, bool)> {
        QuadTreeIterator::new(self, position, theta)
    }
}

struct QuadTreeIteratorVisitData<'a> {
    curr_depth: usize,
    curr_cell: &'a mut usize,
    max_cell: &'a mut usize,
}

struct QuadTreeIteratorVisitStack {
    visit_stack: Vec<(usize, usize)>,
}

impl QuadTreeIteratorVisitStack {
    fn new() -> Self {
        let mut visit_stack = Vec::with_capacity(QuadTreeUtils::MAX_DEPTH);
        visit_stack.push((0, 1));

        Self { visit_stack }
    }

    #[inline(never)]
    fn top_mut<'a>(&'a mut self) -> Option<QuadTreeIteratorVisitData<'a>> {
        let stack_size = self.visit_stack.len();

        self.visit_stack.last_mut()
            .map(|(curr_cell, max_cell)| QuadTreeIteratorVisitData {
                curr_depth: stack_size - 1,
                curr_cell,
                max_cell
            })
    }

    #[inline(never)]
    fn pop(&mut self) {
        self.visit_stack.pop();
    }

    #[inline(never)]
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
    pub fn new(quad_tree: &'a QuadTree<T>, position: Vec2, theta: f32,) -> Self {
        Self {
            position,
            theta,
            quad_tree,
            visit_stack: QuadTreeIteratorVisitStack::new(),
            nearest_cells: (0..quad_tree.depth).map(|depth|
                QuadTreeUtils::nearest_cell_at_layer(depth, position)
            ).collect()
        }
    }

    /// Get cell of relative `index` at `depth`.
    #[inline(never)]
    fn cell_at_depth(&self, depth: usize, index: usize) -> &'a T {
        let offset = QuadTreeUtils::layer_end(depth);
        &self.quad_tree.cells[offset + index - 1]
    }
}

impl<'a, T> Iterator for QuadTreeIterator<'a, T> {
    type Item = (&'a T, bool);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(QuadTreeIteratorVisitData {
            curr_depth,
            curr_cell,
            max_cell,
        }) = self.visit_stack.top_mut() {
            // go up one level if the whole subtree has been visited
            if curr_cell >= max_cell {
                self.visit_stack.pop();
                continue;
            }

            // visit subtree
            let curr_width = QuadTreeUtils::layer_width(curr_depth);
            let curr_position = QuadTreeUtils::position_at_layer(curr_depth, *curr_cell);
            let ratio: f32 = curr_width / self.position.metric_distance(&curr_position);
            if ratio > self.theta || curr_depth >= (self.quad_tree.depth - 1) {
                // return current cell
                let curr_cell_old = *curr_cell;
                let belongs_to_cell = *curr_cell == self.nearest_cells[curr_depth];
                *curr_cell += 1;
                return Some((self.cell_at_depth(curr_depth, curr_cell_old), belongs_to_cell));
            } else {
                // visit subtree
                let curr_cell_subtree = *curr_cell * 4;
                *curr_cell += 1;
                self.visit_stack.push(curr_cell_subtree);
            }
        }

        None
    }
}

pub struct QuadTreeUtils;

impl QuadTreeUtils {
    const MAX_DEPTH: usize = 10;

    /// Number of cells for a tree of the given `depth`.
    #[inline(never)]
    fn tree_size(depth: usize) -> usize {
        Self::layer_end(depth)
    }

    /// Number of cells at the given `depth`.
    #[inline(never)]
    fn layer_size(depth: usize) -> usize {
        1 << (2 * depth)
    }

    /// Number of cells per dimension at given `depth`:
    /// `layer_dim(x) * layer_dim(x) == layer_size(x)`
    #[inline(never)]
    fn layer_dim(depth: usize) -> usize {
        1 << depth
    }

    /// Width of one cell at the given `depth`.
    #[inline(never)]
    fn layer_width(depth: usize) -> f32 {
        1.0 / Self::layer_dim(depth) as f32
    }

    /// Start offset of the given layer in the whole array.
    #[inline(never)]
    fn layer_start(depth: usize) -> usize {
        let mut size = 0;
        let mut tmp = 1;
        for _ in 0..depth {
            size += tmp;
            tmp *= 4;
        }

        size
    }

    /// End (excluded) offset of the given layer in the whole array.
    #[inline(never)]
    fn layer_end(depth: usize) -> usize {
        Self::layer_start(depth + 1)
    }

    /// Relative nearest cell to `normalized_pos` at `depth`.
    #[inline(never)]
    fn nearest_cell_at_layer(depth: usize, normalized_pos: Vec2) -> usize {
        if depth == 0 {
            return 0;
        }

        let layer_dim = QuadTreeUtils::layer_dim(depth);
        let layer_width = QuadTreeUtils::layer_width(depth);
        let up_layer_dim = layer_dim / 2;

        let up_cell = (normalized_pos * up_layer_dim as f32).map(|e| e.floor());
        let up_row_offset = up_cell.y as usize;
        let up_col_offset = up_cell.x as usize;

        // Values should be between 0.0 and 2.0
        let in_layer_pos = (normalized_pos - up_cell) / layer_width;

        up_row_offset * 4 + up_col_offset
            +
        Self::nearest_cell_in_layer(in_layer_pos)
    }

    #[inline(never)]
    /// Inverse of `nearest_cell_in_layer`
    fn nearest_cell_in_layer(pos: Vec2) -> usize {
        if pos.x <= 1.0 {
            if pos.y <= 1.0 {
                0
            } else {
                1
            }
        } else {
            if pos.y <= 1.0 {
                2
            } else {
                3
            }
        }
    }

    /// Nearest cell position to relative cell of `index` at `depth`.
    #[inline(never)]
    fn position_at_layer(depth: usize, index: usize) -> Vec2 {
        if depth == 0 {
            return Vec2::new(0.5, 0.5);
        }

        let up_row_offset = (index / 4) / 4;
        let up_col_offset = (index / 4) % 4;
        let cell_offset = index % 4;

        let layer_width = QuadTreeUtils::layer_width(depth);
        let up_layer_width = layer_width * 2.0;

        Vec2::new(up_col_offset as f32, up_row_offset as f32) * up_layer_width
            +
        Self::position_in_layer(cell_offset) * layer_width
    }

    #[inline(never)]
    /// Inverse of `nearest_cell_in_layer`
    fn position_in_layer(index: usize) -> Vec2 {
        const OFFSETS: [Vec2; 4] = [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
        ];

        OFFSETS[index]
    }
}

impl ExtraParams {
    pub fn from_bodies(bodies: &[Body]) -> ExtraParams {
        ExtraParams {
            min: *bodies.iter().map(|body| ordered_float::OrderedFloat(body.position.x.min(body.position.y))).min().unwrap(),
            max: *bodies.iter().map(|body| ordered_float::OrderedFloat(body.position.x.max(body.position.y))).max().unwrap(),
            min_distance: 1.0,
        }
    }

    #[inline(never)]
    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    #[inline(never)]
    pub fn normalize(&self, vec: Vec2) -> Vec2 {
        vec / self.size()
    }

    #[inline(never)]
    pub fn max_depth(&self) -> usize {
        (self.size() / self.min_distance).log2().max(1.0).ceil() as usize
    }
}