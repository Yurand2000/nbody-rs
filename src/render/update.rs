use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use crate::render::scenes::SimulationUiEntities;
use crate::sim::Simulator as _;

use super::*;

pub fn update(
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
    bodies: Query<(&mut Transform, &BodyId)>,
) {
    nbody.step();
    nbody.pull_updates();

    for (mut transform, id) in bodies {
        let body = nbody.get_body(id.0);

        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
    }
}

pub fn process_inputs(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    state: Res<State<render::SimUpdateState>>,
    mut next_state: ResMut<NextState<render::SimUpdateState>>,
    mut next_menu_state: ResMut<NextState<render::SimMenuState>>,
    mut camera: Query<(&mut Transform, &Camera2d), With<SimEntityTag>>,
) {
    use render::SimUpdateState::*;
    use bevy::input::keyboard::Key::*;

    if keys.just_pressed(Escape) {
        next_menu_state.set(render::SimMenuState::EscMenu);
        return;
    }

    if keys.just_pressed(Space) {
        match *state.get() {
            Running => next_state.set(Paused),
            Paused => next_state.set(Running),
        }
    }

    let (mut camera, _) = camera.single_mut().unwrap();

    let mut mov = Vec3::ZERO;
    if keys.pressed(Character("w".into())) {
        mov.y += 1.0;
    }

    if keys.pressed(Character("a".into())) {
        mov.x -= 1.0;
    }

    if keys.pressed(Character("s".into())) {
        mov.y -= 1.0;
    }

    if keys.pressed(Character("d".into())) {
        mov.x += 1.0;
    }

    camera.translation += mov * 10.0;

    for evt in mouse_wheel.read() {
        if evt.y > 0.0 {
            camera.scale.x /= 1.2;
            camera.scale.y /= 1.2;
        } else {
            camera.scale.x *= 1.2;
            camera.scale.y *= 1.2;
        }
    }
}

pub fn process_inputs_simulate(
    keys: Res<ButtonInput<bevy::input::keyboard::Key>>,
    mut nbody: ResMut<sim::async_simulator::AsyncSimulatorFrontend>,
    ui_entities: Res<SimulationUiEntities>,
    mut text_query: Query<&mut Text>,
) {
    let mut params = nbody.get_params();

    if keys.pressed(bevy::input::keyboard::Key::Character("+".into())) &&
       !keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.time_step *= 1.05;
    }

    if keys.pressed(bevy::input::keyboard::Key::Character("-".into())) &&
       !keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.time_step /= 1.05;
    }

    if keys.just_pressed(bevy::input::keyboard::Key::Character("+".into())) &&
       keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.theta *= 2.0;
    }

    if keys.just_pressed(bevy::input::keyboard::Key::Character("-".into())) &&
       keys.pressed(bevy::input::keyboard::Key::Shift)
    {
        params.theta /= 2.0;
    }

    nbody.set_params(params);

    let mut time_step_text = text_query.get_mut(ui_entities.time_step_text).unwrap();
    *time_step_text = Text::new(format!("Time Step: {:.3}", params.time_step));

    let mut theta_text = text_query.get_mut(ui_entities.theta_text).unwrap();
    *theta_text = Text::new(format!("Theta: {:.3}", params.theta));
}

pub fn update_fps_text(
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    ui_entities: Res<SimulationUiEntities>,
    mut text_query: Query<&mut Text>,
) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
       && let Some(value) = fps.smoothed()
    {
        let mut fps_text = text_query.get_mut(ui_entities.fps_text).unwrap();
        *fps_text = Text::new(format!("FPS: {:.2}", value));
    }
}

