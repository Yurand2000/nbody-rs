use super::*;

#[derive(Debug, Clone)]
pub struct QuadTree<T> {
    pub cells: Vec<T>,
    pub depth: usize,
    pub norm: NormalizationData,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NormalizationData {
    pub offset: Vec2f,
    pub size: Vec2f,
    pub min_distance: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct QuadTreeCell {
    pub mass: f32,
    pub position: Vec2f,
}

impl<T> QuadTree<T> {
    pub fn new(mut depth: usize, norm: NormalizationData) -> Self
        where T: Default + Clone
    {
        // max depth
        if depth > QuadTreeUtils::MAX_DEPTH_CPU {
            depth = QuadTreeUtils::MAX_DEPTH_CPU;
        }

        Self {
            cells: vec![T::default(); QuadTreeUtils::tree_size(depth)],
            depth,
            norm,
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
    pub fn get_interactions(&self, position: Vec2f, theta: f32) -> impl Iterator<Item = (&QuadTreeCell, bool)> {
        QuadTreeIterator::new(self, position, theta)
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
    position: Vec2f,
    theta: f32,
    quad_tree: &'a QuadTree<T>,
    visit_stack: QuadTreeIteratorVisitStack,
    nearest_cells: Vec<usize>,
}

impl<'a, T> QuadTreeIterator<'a, T> {
    pub fn new(quad_tree: &'a QuadTree<T>, position: Vec2f, theta: f32) -> Self {
        Self {
            position,
            theta,
            quad_tree,
            visit_stack: QuadTreeIteratorVisitStack::new(),
            nearest_cells: (0 ..= quad_tree.depth).map(|depth|
                QuadTreeUtils::nearest_cell_at_layer(depth, quad_tree.norm.normalize(position))
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
            let ratio: f32 = curr_width / (self.position - cell.position).magnitude() * self.quad_tree.norm.size.magnitude();
            if ratio < self.theta || curr_depth >= self.quad_tree.depth {
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
pub struct QuadTreeUtils;

impl QuadTreeUtils {
    pub const MAX_DEPTH_CPU: usize = 10;
    pub const MAX_DEPTH_GPU: usize = 15;

    /// Number of cells for a tree of the given `depth`.
    #[inline(always)]
    pub fn tree_size(depth: usize) -> usize {
        Self::layer_end(depth)
    }

    /// Number of cells at the given `depth`.
    #[inline(always)]
    pub fn layer_size(depth: usize) -> usize {
        1 << (2 * depth)
    }

    /// Number of cells per dimension at given `depth`:
    /// `layer_dim(x) * layer_dim(x) == layer_size(x)`
    #[inline(always)]
    pub fn layer_dim(depth: usize) -> usize {
        1 << depth
    }

    /// Width of one cell at the given `depth`.
    #[inline(always)]
    pub fn layer_width(depth: usize) -> f32 {
        1.0 / Self::layer_dim(depth) as f32
    }

    /// Start offset of the given layer in the whole array.
    #[inline(always)]
    pub fn layer_start(depth: usize) -> usize {
        let mut size = 0;
        let mut tmp = 1;
        for _ in 0..depth {
            size += tmp;
            tmp *= 4;
        }

        size
    }

    /// End (excluded) offset of the given layer in the whole array.
    #[inline(always)]
    pub fn layer_end(depth: usize) -> usize {
        Self::layer_start(depth + 1)
    }

    /// Relative nearest cell to `normalized_pos` at `depth`.
    #[inline(always)]
    pub fn nearest_cell_at_layer(depth: usize, normalized_pos: Vec2f) -> usize {
        let mut out = 0;
        let mut layer_dim = QuadTreeUtils::layer_dim(1) as f32;
        let mut pos = normalized_pos;
        for _ in 0 .. depth {
            let cell = Self::nearest_cell_in_layer(pos * layer_dim);
            out = Self::first_position_at_next_layer(out) + cell;
            pos -= Self::position_in_layer(cell) / layer_dim;
            layer_dim *= 2.0;
        }

        out
    }

    #[inline(always)]
    /// Inverse of `nearest_cell_in_layer`
    pub fn nearest_cell_in_layer(pos: Vec2f) -> usize {
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
    #[inline(always)]
    pub fn position_at_layer(depth: usize, index: usize) -> Vec2f {
        let mut out = Vec2f::zeros();
        let mut curr_idx = index;
        let mut curr_size = Self::layer_width(depth);
        for _ in 0 .. depth {
            out += Self::position_in_layer(curr_idx % 4) * curr_size;
            curr_size *= 2.0;
            curr_idx = curr_idx / 4;
        }

        out
    }

    #[inline(always)]
    /// Inverse of `nearest_cell_in_layer`
    pub fn position_in_layer(index: usize) -> Vec2f {
        const OFFSETS: [Vec2f; 4] = [
            Vec2f::new(0.0, 0.0),
            Vec2f::new(0.0, 1.0),
            Vec2f::new(1.0, 0.0),
            Vec2f::new(1.0, 1.0),
        ];

        OFFSETS[index]
    }

    #[inline(always)]
    pub fn first_position_at_next_layer(index: usize) -> usize {
        index * 4
    }
}

impl NormalizationData {
    pub fn new(bodies: &[Body]) -> NormalizationData {
        let mut min = Vec2f::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vec2f::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

        for Body { position, .. } in bodies {
            min.x = min.x.min(position.x);
            min.y = min.y.min(position.y);
            max.x = max.x.max(position.x);
            max.y = max.y.max(position.y);
        }

        // Proportional to the extent (1.0 if degenerate)
        let mut delta_x = 0.01 * (max.x - min.x);
        let mut delta_y = 0.01 * (max.y - min.y);
        if delta_x <= 0.0 { delta_x = 1.0; }
        if delta_y <= 0.0 { delta_y = 1.0; }

        max += Vec2f::new(delta_x, delta_y);
        min -= Vec2f::new(delta_x, delta_y);

        NormalizationData {
            offset: min,
            size: max - min,
            min_distance: 1.0,
        }
    }

    #[inline(always)]
    pub fn normalize(&self, vec: Vec2f) -> Vec2f {
        let normalized = (vec - self.offset).component_div(&self.size);
        let normalized = Vec2f::new(normalized.x, 1.0 - normalized.y);

        debug_assert!(
            normalized.x >= 0f32 && normalized.y >= 0f32 &&
            normalized.x < 1f32 && normalized.y < 1f32,
            "{:?} {:?} {:?} {:?}", vec, normalized, self.offset, self.size);

        normalized
    }

    #[inline(always)]
    pub fn invert_normalize(&self, vec: Vec2f) -> Vec2f {
        let vec = Vec2f::new(vec.x, 1.0 - vec.y);
        vec.component_mul(&self.size) + self.offset
    }

    #[inline(always)]
    pub fn invert_normalize_size(&self, vec: Vec2f) -> Vec2f {
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
