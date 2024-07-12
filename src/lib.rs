use bevy::ecs::system::SystemId;
use bevy::prelude::*;
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
    pub commands_args: HashMap<String, SystemId<Vec<String>, ()>>,
    pub commands_noargs: HashMap<String, SystemId<(), ()>>,
}

/// Provides methods for managing the available "console commands"
///
/// There are two kinds of commands:
///  - "args" (system takes `Vec<String>` as input)
///  - "noargs" (system takes no input)
///
/// When you try to run a cli string, the correct variant will be chosen based on whether
/// args were present.
///
/// You should be able to do this at `App` creation, as well as later at
/// runtime, with exclusive `World` access.
pub trait CliCommandsRegisterExt {
    /// Create a new "console command" with the given string and system/implementation
    ///
    /// If a command with the same name already exists, it is replaced.
    fn register_clicommand_noargs<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<(), (), Param> + 'static;
    /// Create a new "console command" with the given string and system/implementation
    ///
    /// If a command with the same name already exists, it is replaced.
    fn register_clicommand_args<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<Vec<String>, (), Param> + 'static;
    /// Remove a "console command", if it exists
    ///
    /// `has_args` specifies whether you want to remove the "args" or "noargs" variant.
    fn deregister_clicommand(&mut self, name: &str, has_args: bool) -> &mut Self;
}

/// Provides methods to run/call "console commands"
///
/// You should be able to do this with exclusive `World` access, or using Bevy's `Commands`.
pub trait CliCommandsRunExt {
    fn run_clicommand(&mut self, name: &str);
}

impl CliCommandsRegisterExt for World {
    fn register_clicommand_noargs<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<(), (), Param> + 'static,
    {
        self.deregister_clicommand(name, false);
        self.init_resource::<CliCommands>();
        let id = self.register_system(system);
        self.resource_mut::<CliCommands>()
            .commands_noargs
            .insert(name.to_owned(), id);
        self
    }
    fn register_clicommand_args<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<Vec<String>, (), Param> + 'static,
    {
        self.deregister_clicommand(name, true);
        self.init_resource::<CliCommands>();
        let id = self.register_system(system);
        self.resource_mut::<CliCommands>()
            .commands_args
            .insert(name.to_owned(), id);
        self
    }
    fn deregister_clicommand(&mut self, name: &str, has_args: bool) -> &mut Self {
        let Some(mut clicommands) = self.get_resource_mut::<CliCommands>() else {
            return self;
        };
        if has_args {
            clicommands.commands_args.remove(name);
        } else {
            clicommands.commands_noargs.remove(name);
        }
        self
    }
}

impl CliCommandsRegisterExt for App {
    fn register_clicommand_noargs<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<(), (), Param> + 'static,
    {
        self.world_mut().register_clicommand_noargs(name, system);
        self
    }
    fn register_clicommand_args<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<Vec<String>, (), Param> + 'static,
    {
        self.world_mut().register_clicommand_args(name, system);
        self
    }
    fn deregister_clicommand(&mut self, name: &str, has_args: bool) -> &mut Self {
        self.world_mut().deregister_clicommand(name, has_args);
        self
    }
}

impl CliCommandsRunExt for World {
    fn run_clicommand(&mut self, cli: &str) {
        // TODO: support quotes and other such fancy syntax?
        let mut iter = cli.split_ascii_whitespace();
        let Some(name) = iter.next() else {
            error!("Attempted to run empty CLI string!");
            return;
        };
        let args: Vec<String> = iter.map(|s| s.to_owned()).collect();

        if !args.is_empty() {
            let Some(id) = self
                .resource::<CliCommands>()
                .commands_args
                .get(name)
            else {
                error!("CliCommand {:?} not found!", name);
                return;
            };
            debug!("Running CliCommand {:?}, args: {:?}", name, args);
            if let Err(e) = self.run_system_with_input(id.clone(), args) {
                error!("CliCommand {:?} failed to run: {}", name, e);
            }
        } else {
            let Some(id) = self
                .resource::<CliCommands>()
                .commands_noargs
                .get(name)
            else {
                warn!("CliCommand {:?} not found!", name);
                return;
            };
            debug!("Running CliCommand {:?} (noargs)", name);
            if let Err(e) = self.run_system(id.clone()) {
                error!("CliCommand {:?} failed to run: {}", name, e);
            }
        }
    }
}

impl CliCommandsRunExt for App {
    fn run_clicommand(&mut self, name: &str) {
        self.world_mut().run_clicommand(name);
    }
}

impl<'w, 's> CliCommandsRunExt for Commands<'w, 's> {
    fn run_clicommand(&mut self, name: &str) {
        self.add(CliRunCommand(name.to_owned()));
    }
}

pub struct CliRunCommand(pub String);

impl bevy::ecs::world::Command for CliRunCommand {
    fn apply(self, world: &mut World) {
        world.run_clicommand(&self.0);
    }
}
