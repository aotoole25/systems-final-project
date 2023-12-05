use fyrox::{
    core::{
        algebra::{Vector2, Vector3},
        futures::executor::block_on,
        pool::Handle,
        reflect::prelude::*,
        uuid::{uuid, Uuid},
        visitor::prelude::*, TypeUuidProvider
    },
    asset::manager::ResourceManager,
    event::{ElementState, Event, WindowEvent},
    keyboard::KeyCode,
    impl_component_provider,
    resource::texture::Texture,
    gui::message::UiMessage,
    plugin::{Plugin, PluginConstructor, PluginContext, PluginRegistrationContext},
    scene::{
        dim2::{rectangle::Rectangle, rigidbody::RigidBody},
        node::{Node},
        Scene, SceneLoader,
    },
    script::{ScriptContext, ScriptTrait},
};
use std::path::Path;

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct Player{
    sprite: Handle<Node>,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
}

impl_component_provider!(Player,);

impl TypeUuidProvider for Player {
    // Returns unique script id for serialization needs.
    fn type_uuid() -> Uuid {
        uuid!("c5671d19-9f1a-4286-8486-add4ebaadaec")
    }
}

impl ScriptTrait for Player {
    // Called once at initialization.
    fn on_init(&mut self, context: &mut ScriptContext) {    }
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) { }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::KeyboardInput { event, .. } = event {
                let is_pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    fyrox::keyboard::PhysicalKey::Code(KeyCode::KeyA) => self.move_left = is_pressed,
                    fyrox::keyboard::PhysicalKey::Code(KeyCode::KeyD) => self.move_right = is_pressed,
                    fyrox::keyboard::PhysicalKey::Code(KeyCode::KeyW) => self.move_up = is_pressed,
                    fyrox::keyboard::PhysicalKey::Code(KeyCode::KeyS) => self.move_down = is_pressed,
                    _ => (),
                }
            }
        }
    }

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        // The script can be assigned to any scene node, but we assert that it will work only with
        // 2d rigid body nodes.
        if let Some(rigid_body) = context.scene.graph[context.handle].cast_mut::<RigidBody>() {
            let x_speed = match (self.move_left, self.move_right) {
                (true, false) => 3.0,
                (false, true) => -3.0,
                _ => 0.0,
            };
            let y_speed = match (self.move_up, self.move_down) {
                (true, false) => 3.0,
                (false, true) => -3.0,
                _ => 0.0,
            };

            rigid_body.set_lin_vel(Vector2::new(x_speed, y_speed));
        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}
