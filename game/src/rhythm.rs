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
    event::{ElementState, Event, WindowEvent, MouseButton},
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
pub struct RhythmBlock{
    speed: f32,
    reference_block: Handle<Node>,
    good: bool,
    great: bool,
    touching: bool, 
    clicked: bool,
}

const GOOD_DISTANCE: f32 = 0.2;
const GREAT_DISTANCE: f32 = 0.1;
const TOUCH_DISTANCE: f32 = 1.0;

impl_component_provider!(RhythmBlock,);

impl TypeUuidProvider for RhythmBlock {
    // Returns unique script id for serialization needs.
    fn type_uuid() -> Uuid {
        uuid!("e59a3ea3-fbaa-4b7d-be15-ed2388dd6c72")
    }
}

impl ScriptTrait for RhythmBlock {
    // Called once at initialization.
    fn on_init(&mut self, context: &mut ScriptContext) { }
    
    // Put start logic - it is called when every other script is already initialized.
    fn on_start(&mut self, context: &mut ScriptContext) {
        self.touching = false;
        self.good = false;
        self.great = false;
        self.clicked = false;
    }

    // Called whenever there is an event from OS (mouse click, keypress, etc.)
    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        if let Event::WindowEvent { event, .. } = event {
            if let &WindowEvent::MouseInput {button, state, .. } = event {
                if button == MouseButton::Left {
                    self.clicked = state == ElementState::Pressed;
                }
            }
        }
    }

    // Called every frame at fixed rate of 60 FPS.
    fn on_update(&mut self, context: &mut ScriptContext) {
        if let Some(ref_block) = context.scene.graph.try_get(self.reference_block){
            //gets position of reference block
            let ref_position = ref_block.local_transform().position().clone();

            //script only works for rectangles
            if let Some(rectangle) = context.scene.graph[context.handle].cast_mut::<Rectangle>() {
                //moves block consistently downwards at a rate corresponding to its speed
                let transform = rectangle.local_transform_mut();
                let offset_down = Vector3::new(0.0, self.speed*-0.01, 0.0);
                transform.offset(offset_down);

                //touching reference block
                if self.touching == false && (rectangle.local_transform().position().y <= ref_position.y + TOUCH_DISTANCE) {
                    self.touching = true;
                }

                //close enough to ref block to be "good"
                if self.good == false && (rectangle.local_transform().position().y <= ref_position.y + GOOD_DISTANCE) {
                    self.good = true;
                }

                //close enough to be "great"
                if self.great == false && (rectangle.local_transform().position().y <= ref_position.y + GREAT_DISTANCE) {
                    self.good = true;
                }

                //farther and no longer great
                if self.great == true && (rectangle.local_transform().position().y <= ref_position.y - GREAT_DISTANCE) {
                    self.great = false;
                }

                //farther and no longer "good"
                if self.good == true && (rectangle.local_transform().position().y <= ref_position.y - GOOD_DISTANCE) {
                    self.good = false;
                }

                //farther and no longer "touching"
                if self.touching == true && (rectangle.local_transform().position().y <= ref_position.y - TOUCH_DISTANCE) {
                    self.touching = false;
                }

                if self.clicked == true {
                    rectangle.set_visibility(false);
                }
            }
        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}
