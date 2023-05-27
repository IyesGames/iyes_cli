use bevy::prelude::*;
use bevy::ecs::system::BoxedSystem;
use bevy::utils::HashMap;

pub mod prelude {
    pub use crate::{CliCommandsRegisterExt, CliCommandsRunExt};
}

/// Stores all the known/available commands that can be called.
///
/// This resource contains, for each command, the system that implements it,
/// and any metadata needed for it to be run, such as its name string.
///
/// Since all your commands are stored in this resource, they are per-World.
#[derive(Resource, Default)]
pub struct CliCommands {
    commands: HashMap<String, Option<BoxedSystem>>,
}

/// Provides methods for managing the available "console commands"
///
/// You should be able to do this at `App` creation, as well as later at
/// runtime, with exclusive `World` access.
pub trait CliCommandsRegisterExt {
    /// Create a new "console command" with the given string and system/implementation
    ///
    /// If a command with the same name already exists, it is replaced.
    fn register_clicommand<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where S: IntoSystem<(), (), Param>;
    /// Remove a "console command", if it exists
    fn deregister_clicommand(&mut self, name: &str) -> &mut Self;
}

/// Provides methods to run/call "console commands"
///
/// You should be able to do this with exclusive `World` access, or using Bevy's `Commands`.
pub trait CliCommandsRunExt {
    fn run_clicommand(&mut self, name: &str);
}

impl CliCommandsRegisterExt for World {
    fn register_clicommand<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where S: IntoSystem<(), (), Param> {
        self.deregister_clicommand(name);
        self.init_resource::<CliCommands>();
        let mut system = IntoSystem::into_system(system);
        system.initialize(self);
        self.resource_mut::<CliCommands>().commands
            .insert(name.to_owned(), Some(Box::new(system)));
        self
    }
    fn deregister_clicommand(&mut self, name: &str) -> &mut Self {
        let Some(mut clicommands) = self.get_resource_mut::<CliCommands>() else {
            return self;
        };
        clicommands.commands.remove(name);
        self
    }
}

impl CliCommandsRegisterExt for App {
    fn register_clicommand<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where S: IntoSystem<(), (), Param> {
        self.world.register_clicommand(name, system);
        self
    }
    fn deregister_clicommand(&mut self, name: &str) -> &mut Self {
        self.world.deregister_clicommand(name);
        self
    }
}

impl CliCommandsRunExt for World {
    fn run_clicommand(&mut self, name: &str) {
        let Some(mut system) = self.resource_mut::<CliCommands>().bypass_change_detection()
            .commands.get_mut(name).and_then(|opt| opt.take())
        else {
            warn!("CliCommand {:?} not found!", name);
            return;
        };
        info!("Running CliCommand {:?}", name);
        system.run((), self);
        system.apply_buffers(self);
        if let Some(cmd) = self.resource_mut::<CliCommands>().bypass_change_detection()
            .commands.get_mut(name)
        {
            *cmd = Some(system);
        }
    }
}

impl CliCommandsRunExt for App {
    fn run_clicommand(&mut self, name: &str) {
        self.world.run_clicommand(name);
    }
}

impl<'w, 's> CliCommandsRunExt for Commands<'w, 's> {
    fn run_clicommand(&mut self, name: &str) {
        self.add(CliRunCommand(name.to_owned()));
    }
}

pub struct CliRunCommand(pub String);

impl bevy::ecs::system::Command for CliRunCommand {
    fn write(self, world: &mut World) {
        world.run_clicommand(&self.0);
    }
}
