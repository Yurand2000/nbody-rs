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
    state: Res<State<render::SimPausedState>>,
    mut next_state: ResMut<NextState<render::SimPausedState>>,
) {
    use render::SimPausedState::*;

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowRight) {
        match *state.get() {
            Paused => next_state.set(DebugQuadTree),
            DebugQuadTree => next_state.set(DebugForces),
            DebugForces => next_state.set(DebugForceSingle),
            DebugForceSingle => next_state.set(Paused),
        }
    }

    if keys.just_pressed(bevy::input::keyboard::Key::ArrowLeft) {
        match *state.get() {
            Paused => next_state.set(DebugForceSingle),
            DebugQuadTree => next_state.set(Paused),
            DebugForces => next_state.set(DebugQuadTree),
            DebugForceSingle => next_state.set(DebugForces),
        }
    }
}
