use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{canvas::Canvas, layer::Layer, Timeline};

#[derive(Debug)]
pub enum Command {
    Paint(Paint),
    AddLayer(AddLayer),
    DeleteLayer,
}

impl Command {
    pub fn paint(buffer: Vec<u8>) -> Self {
        Command::Paint(Paint { buffer })
    }
    pub fn process(&mut self, world: &mut World) {
        use Command::*;
        match self {
            Paint(ref mut c) => c.process(world),
            AddLayer(ref mut c) => c.process(world),
            _ => todo!(),
        }
    }
    pub fn undo(&mut self, world: &mut World) {
        use Command::*;
        match *self {
            Paint(ref mut c) => c.undo(world),
            AddLayer(ref mut c) => c.undo(world),
            _ => todo!(),
        }
    }
}

trait CanvasCommand {
    fn process(&mut self, world: &mut World);

    fn undo(&mut self, world: &mut World);
}

#[derive(Debug)]
pub struct AddLayer {}

impl CanvasCommand for AddLayer {
    fn process(&mut self, world: &mut World) {
        todo!()
    }

    fn undo(&mut self, world: &mut World) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Paint {
    buffer: Vec<u8>,
}

impl Paint {
    fn do_stuff(&mut self, world: &mut World) {
        world.resource_scope(|world, canvas: Mut<Canvas>| {
            world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
                let mut layers = world.query::<(Entity, &Layer)>();

                let (_, layer) = layers.get(world, canvas.layer_id).unwrap();
                let image = images.get_mut(&layer.frames[&0]).unwrap();

                assert_eq!(image.data.len(), self.buffer.len());
                std::mem::swap(&mut image.data, &mut self.buffer);
            });
        });
    }
}

impl CanvasCommand for Paint {
    fn process(&mut self, world: &mut World) {
        self.do_stuff(world);
        info!("COMMAND: paint");
    }

    fn undo(&mut self, world: &mut World) {
        self.do_stuff(world);
        info!("COMMAND: undo paint");
    }
}

#[derive(Resource, Default)]
pub struct CanvasCommands {
    queue: VecDeque<Command>,
    history: Vec<Command>,
    undo_history: Vec<Command>,
    max_history: Option<usize>,
}

impl CanvasCommands {
    pub fn add(&mut self, command: Command) {
        self.queue.push_back(command)
    }

    pub fn paint(&mut self, buffer: Vec<u8>) {
        self.add(Command::paint(buffer));
    }

    fn call(&mut self, world: &mut World) {
        while let Some(mut command) = self.queue.pop_front() {
            command.process(world);
            self.history.push(command);
        }
    }

    pub fn undo(&mut self, world: &mut World) -> bool {
        if let Some(mut command) = self.history.pop() {
            command.undo(world);
            self.undo_history.push(command);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, world: &mut World) -> bool {
        if let Some(mut command) = self.undo_history.pop() {
            command.process(world);
            self.history.push(command);
            true
        } else {
            false
        }
    }
}

pub fn process_commands(world: &mut World) {
    world.resource_scope(|world, mut canvas_commands: Mut<CanvasCommands>| {
        canvas_commands.call(world)
    })
}
