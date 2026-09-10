use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(nbody_rs::GamePlugin)
        .run();
}
