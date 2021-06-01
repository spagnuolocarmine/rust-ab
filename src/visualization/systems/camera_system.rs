use bevy::app::Events;
use bevy::prelude::{Query, Transform, Window, WindowDescriptor};
use bevy::window::WindowResized;

use crate::bevy::prelude::Res;
use crate::bevy::render::camera::Camera;
use crate::visualization::simulation_descriptor::SimulationDescriptor;

pub fn camera_system(
    resize_event: Res<Events<WindowResized>>,
    sim: Res<SimulationDescriptor>,
    mut query: Query<(&Camera, &mut Transform)>,
) {
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        if let Ok((_camera, mut transform)) = query.single_mut() {
            /*let window_width = e.width - 300.;
            let ww_translation = e.width;
            transform.translation.x =
                ((ww_translation * 0.5) - (ww_translation - sim.width as f32) / 2.) * 0.5;
            transform.translation.y = (e.height * 0.5) - (e.height - sim.height as f32) / 2.;
            transform.scale.x = sim.width / window_width;
            transform.scale.y = sim.height / e.height;*/

            let window_width = e.width;
            let scale_x = sim.width / e.width;
            // 0 is the center of the window, which changes when the window resizes
            /*let ww_translation = (e.width * 0.5) // Left-most point in view of the window
                - 150. // Offset to take the side panel in account
            - (e.width - 150.) * 0.5; // center it in the right side of the window*/

            // THIS WORKS but it fills the screen
            /*let ww_translation = 100.;
            transform.translation.x = ww_translation; //((ww_translation * 0.5) - (ww_translation - sim.width as f32) / 2.) - 60.;
            transform.translation.y = 100.; //(e.height * 0.5) - (e.height - sim.height as f32) / 2.;
            transform.scale.x = sim.width / e.width;
            transform.scale.y = sim.height / e.height;*/

            // Test with coordinate origin set to bottom left: it seems to work correctly but the scale fails to work because the translation x should stay constant
            // with an offset of -75 regardless of windows size?
            /*let ww_translation = 110.;
            transform.translation.x = -75.; //ww_translation; //((ww_translation * 0.5) - (ww_translation - sim.width as f32) / 2.) - 60.;
            transform.translation.y = 0.; //100.; //(e.height * 0.5) - (e.height - sim.height as f32) / 2.;
            transform.scale.x = sim.width / (e.width - 350.);
            transform.scale.y = sim.height / e.height;*/

            let ui_offset = -sim.ui_width;
            let scale_x = sim.width / (e.width + ui_offset);

            transform.translation.x = ui_offset * scale_x; //ww_translation; //((ww_translation * 0.5) - (ww_translation - sim.width as f32) / 2.) - 60.;
            transform.translation.y = 0.; //100.; //(e.height * 0.5) - (e.height - sim.height as f32) / 2.;
            transform.scale.x = scale_x; //sim.width / (e.width);
            transform.scale.y = sim.height / e.height;

            /*transform.translation.x = 90.; //e.width * aspect_ratio; //(e.width / 100.);
            transform.translation.y = 100.;
            transform.scale.x = (sim.width / e.width) * 2.;
            transform.scale.y = sim.height / e.height;*/
            println!(
                "x = {}, y = {}, scale x = {}, scale y = {}, width = {}, height = {};",
                transform.translation.x,
                transform.translation.y,
                transform.scale.x,
                transform.scale.y,
                e.width,
                e.height
            );
        }
    }
}
