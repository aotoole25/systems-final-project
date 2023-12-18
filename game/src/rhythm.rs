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
    script::{ScriptContext, ScriptTrait, ScriptMessagePayload, ScriptMessageContext},
};
use std::path::Path;

const GOOD_DISTANCE: f32 = 0.2;
const GREAT_DISTANCE: f32 = 0.1;
const TOUCH_DISTANCE: f32 = 1.0;

#[derive(Visit, Reflect, Debug, Clone, Default)]
pub struct RhythmBlock{
    speed: f32,
    reference_block: Handle<Node>,
    good: bool,
    great: bool,
    touching: bool, 
    clicked: bool,
}

enum Message {
    GreatClick,
    GoodClick,
    OkClick,
    BadClick,
    Win,
    Loss,
}

impl_component_provider!(RhythmBlock);

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
            if let WindowEvent::KeyboardInput { event, .. } = event {
                let is_pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    fyrox::keyboard::PhysicalKey::Code(KeyCode::ArrowUp) => self.clicked = event.state == ElementState::Pressed,
                    _ => (),
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

                    if self.great {
                        context.message_sender.send_global(Message::GreatClick);
                    } else if self.good {
                        context.message_sender.send_global(Message::GoodClick);
                    } else if self.touching {
                        context.message_sender.send_global(Message::OkClick);
                    } else {
                        context.message_sender.send_global(Message::BadClick);
                    }
                }
            }
        }
    }

    // Returns unique script ID for serialization needs.
    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}

#[derive(Visit, Reflect, Default, Debug, Clone)]
pub struct ProgressBar {
    progress: f32,
    new_progress: f32,
    move_amount: f32,
    reference_block: Handle<Node>,
}

impl_component_provider!(ProgressBar);

impl TypeUuidProvider for ProgressBar {
    fn type_uuid() -> Uuid {
        uuid!("ba1ccd7e-9def-11ee-8c90-0242ac120002")
    }
}

impl ScriptTrait for ProgressBar {
    fn on_init(&mut self, context: &mut ScriptContext) {
        // Put initialization logic here.
        self.progress = 5.0;
        self.new_progress = 0.0;
        self.move_amount = 0.05;
    }
    
    fn on_start(&mut self, context: &mut ScriptContext) {
        // Put start logic - it is called when every other script is already initialized.
        context.message_dispatcher.subscribe_to::<Message>(context.handle);
    }

    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        // Respond to OS events here.
    }

    fn on_update(&mut self, context: &mut ScriptContext) {
        if let Some(ref_block) = context.scene.graph.try_get(self.reference_block){
            //gets position of reference block
            let ref_position = ref_block.local_transform().position().clone();
            
            //accesses its own rectangle node
            if let Some(rectangle) = context.scene.graph[context.handle].cast_mut::<Rectangle>() {
                //updates if it should go up or down
                if self.new_progress != 0.0 {
                    //moves progress bar
                    let transform = rectangle.local_transform_mut();
                    let y_offset = self.new_progress*self.move_amount;
                    let offset_down = Vector3::new(0.0, y_offset, 0.0);
                    transform.offset(offset_down);

                    //changes progress
                    self.progress += self.new_progress;
                    self.new_progress = 0.0;

                    //checks if win or loss
                    if rectangle.local_transform().position().y >= ref_position.y {
                        context.message_sender.send_global(Message::Win);
                    } else if self.progress <= 0.0 {
                        context.message_sender.send_global(Message::Loss);
                    }
                }
            }
        }
    }

    fn on_message(&mut self, message: &mut dyn ScriptMessagePayload, ctx: &mut ScriptMessageContext,
    ) {
        // React to clicks.
        if let Some(Message::GreatClick) = message.downcast_ref::<Message>() {
            self.new_progress = 4.0;
        }
        if let Some(Message::GoodClick) = message.downcast_ref::<Message>() {
            self.new_progress = 2.0;
        }
        if let Some(Message::OkClick) = message.downcast_ref::<Message>() {
            self.new_progress = 1.0;
        }
        if let Some(Message::BadClick) = message.downcast_ref::<Message>() {
            self.new_progress = -1.0;
        }
    }

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}
