use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    canvas::{Canvas, PaintTool},
    layer::Layer,
    Draw, OperationState,
};

pub mod fill;
pub mod paint;
mod undo;

use self::{
    fill::Fill,
    paint::{Paint, StopPaint},
    undo::{Redo, Undo},
};

// TODO: change name of this enum
pub enum CommandType {
    Command(Box<dyn CanvasCommand>),
    Operation(Box<dyn CanvasOperation>),
}

impl CommandType {
    pub fn command(command: impl CanvasCommand + 'static) -> Self {
        CommandType::Command(Box::new(command))
    }
    pub fn operation(operation: impl CanvasOperation + 'static) -> Self {
        CommandType::Operation(Box::new(operation))
    }
    fn process(&mut self, world: &mut World, canvas_commands: &mut CanvasCommands) {
        match self {
            CommandType::Command(command) => {
                info!("[COMMAND] : {}", command.name());
                command.process(world, canvas_commands)
            }
            CommandType::Operation(command) => {
                info!("[OPERATION] : {}", command.name());
                command.process(world, canvas_commands);
            }
        }
    }
    pub fn is_operation(&self) -> bool {
        match self {
            CommandType::Operation(_) => true,
            CommandType::Command(_) => false,
        }
    }
}

/// a command that can be undone and redone
pub trait CanvasOperation: Send + Sync {
    fn name(&self) -> &'static str;

    fn process(&mut self, world: &mut World, canvas_commands: &mut CanvasCommands);

    fn undo(&mut self, world: &mut World);

    fn cancel(&mut self, world: &mut World) {
        todo!()
    }

    fn redo(&mut self, world: &mut World);
}

/// a basic command that cna not be undone
pub trait CanvasCommand: Send + Sync {
    fn process(&mut self, world: &mut World, canvas_commands: &mut CanvasCommands);

    fn name(&self) -> &'static str;
}

#[derive(Resource, Default)]
pub struct CanvasCommands {
    call_queue: VecDeque<CommandType>,
    cancel_queue: VecDeque<Box<dyn CanvasOperation>>,
    call_history: Vec<Box<dyn CanvasOperation>>,
    undo_history: Vec<Box<dyn CanvasOperation>>,
    max_history: Option<usize>,
}

impl CanvasCommands {
    pub fn add(&mut self, command: CommandType) {
        if let CommandType::Operation(_command) = &command {
            self.undo_history.clear()
        }
        self.call_queue.push_back(command);
    }

    pub fn cancel(&mut self, command: impl CanvasOperation + 'static) {
        self.cancel_queue.push_back(Box::new(command));
    }

    pub fn start_painting(&mut self, color: Color) {
        self.add(CommandType::operation(Paint::new(color)))
    }
    pub fn stop_painting(&mut self) {
        self.add(CommandType::command(StopPaint))
    }

    pub fn fill(&mut self, color: Color) {
        self.add(CommandType::operation(Fill::new(color, 0.5)))
    }

    fn call(&mut self, world: &mut World) {
        while let Some(mut command) = self.call_queue.pop_front() {
            command.process(world, self);

            if let CommandType::Operation(op) = command {
                self.call_history.push(op);
            }
        }

        while let Some(mut command) = self.cancel_queue.pop_front() {
            command.process(world, self);
        }
    }

    pub fn undo(&mut self) {
        self.add(CommandType::command(Undo))
    }

    pub fn redo(&mut self) {
        self.add(CommandType::command(Redo))
    }

    fn perform_undo(&mut self, world: &mut World) {
        if let Some(mut command) = self.call_history.pop() {
            info!("[UNDO] : {}", command.name());
            command.undo(world);
            self.undo_history.push(command);
        }
    }

    fn perform_redo(&mut self, world: &mut World) {
        if let Some(mut command) = self.undo_history.pop() {
            info!("[REDO] : {}", command.name());
            command.redo(world);
            self.call_history.push(command);
        }
    }
}

pub fn process_commands(world: &mut World) {
    world.resource_scope(|world, mut canvas_commands: Mut<CanvasCommands>| {
        canvas_commands.call(world);
    })
}
