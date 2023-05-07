use super::*;

pub struct Undo;

impl CanvasCommand for Undo {
    fn process(&mut self, world: &mut World, canvas_commands: &mut CanvasCommands) {
        canvas_commands.perform_undo(world);
    }

    fn name(&self) -> &'static str {
        "Undo"
    }
}

pub struct Redo;

impl CanvasCommand for Redo {
    fn process(&mut self, world: &mut World, canvas_commands: &mut CanvasCommands) {
        canvas_commands.perform_redo(world);
    }

    fn name(&self) -> &'static str {
        "Redo"
    }
}
