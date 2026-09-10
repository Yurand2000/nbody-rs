use bevy::prelude::*;
use super::*;

pub mod quadtree_debug;
pub mod forces_debug;
pub mod force_single_debug;

pub use quadtree_debug::*;
pub use forces_debug::*;
pub use force_single_debug::*;

pub fn process_inputs_debug(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    state: Res<State<render::SimDebugState>>,
    mut next_state: ResMut<NextState<render::SimDebugState>>,
) {
    use render::SimDebugState::*;

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowRight) {
        match *state.get() {
            None => next_state.set(QuadTree),
            QuadTree => next_state.set(Forces),
            Forces => next_state.set(ForceSingle),
            ForceSingle => next_state.set(None),
        }
    }

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowLeft) {
        match *state.get() {
            None => next_state.set(ForceSingle),
            QuadTree => next_state.set(None),
            Forces => next_state.set(QuadTree),
            ForceSingle => next_state.set(Forces),
        }
    }
}
