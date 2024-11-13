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
    commands: HashMap<String, CliCommandSystems>,
}

struct CliCommandSystems {
    noargs: Option<SystemId<(), ()>>,
    args: Option<SystemId<In<Vec<String>>, ()>>,
}

/// Provides methods for managing the available "console commands"
///
/// A "command" is a Bevy system `fn` identified by a string name.
/// It may take an `In<Vec<String>>` parameter, which can be used to
/// pass in arguments. Or it might not. You can register both kinds
/// of functions for the same command.
///
/// When you try to run a cli string, the correct variant will be
/// chosen based on whether args were present.
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
        S: IntoSystem<In<Vec<String>>, (), Param> + 'static;

    /// Remove a "console command", if it exists
    fn unregister_clicommand(&mut self, name: &str) -> &mut Self;
}

/// Provides methods to run/call "console commands"
///
/// You should be able to do this with exclusive `World` access, or using Bevy's `Commands`.
pub trait CliCommandsRunExt {
    fn run_cli(&mut self, cli: &str);
}

impl CliCommandsRegisterExt for World {
    fn register_clicommand_noargs<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<(), (), Param> + 'static,
    {
        self.init_resource::<CliCommands>();
        let new_id = self.register_system(system);
        let cmds = &mut self.resource_mut::<CliCommands>().commands;
        if let Some(cmd) = cmds.get_mut(name) {
            cmd.noargs = Some(new_id);
        } else {
            cmds.insert(
                name.to_owned(),
                CliCommandSystems {
                    noargs: Some(new_id),
                    args: None,
                },
            );
        }
        self
    }
    fn register_clicommand_args<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<In<Vec<String>>, (), Param> + 'static,
    {
        self.init_resource::<CliCommands>();
        let new_id = self.register_system(system);
        let cmds = &mut self.resource_mut::<CliCommands>().commands;
        if let Some(cmd) = cmds.get_mut(name) {
            cmd.args = Some(new_id);
        } else {
            cmds.insert(
                name.to_owned(),
                CliCommandSystems {
                    args: Some(new_id),
                    noargs: None,
                },
            );
        }
        self
    }
    fn unregister_clicommand(&mut self, name: &str) -> &mut Self {
        let Some(mut clicommands) = self.get_resource_mut::<CliCommands>() else {
            return self;
        };
        clicommands.commands.remove(name);
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
        S: IntoSystem<In<Vec<String>>, (), Param> + 'static,
    {
        self.world_mut().register_clicommand_args(name, system);
        self
    }
    fn unregister_clicommand(&mut self, name: &str) -> &mut Self {
        self.world_mut().unregister_clicommand(name);
        self
    }
}

impl CliCommandsRunExt for World {
    fn run_cli(&mut self, cli: &str) {
        // TODO: support quotes and other such fancy syntax?
        let mut iter = cli.trim().split_ascii_whitespace();

        let Some(name) = iter.next() else {
            error!("Attempted to run empty CLI string!");
            return;
        };

        let args: Vec<String> = iter.map(|s| s.to_owned()).collect();

        let Some(cmd) = self.resource::<CliCommands>().commands.get(name) else {
            error!("CliCommand {:?} not found!", name);
            return;
        };

        if !args.is_empty() {
            if let Some(id) = cmd.args {
                debug!("Running CliCommand {:?} with args: {:?}", name, args);
                if let Err(e) = self.run_system_with_input(id, args) {
                    error!("CliCommand {:?} failed to run: {}", name, e);
                }
                // DONE!
                return;
            } else {
                warn!("CliCommand {:?} does not support args; discarding args!", name);
            }
        }

        if let Some(id) = cmd.noargs {
            debug!("Running CliCommand {:?} (without args)", name);
            if let Err(e) = self.run_system(id) {
                error!("CliCommand {:?} failed to run: {}", name, e);
            }
        } else if let Some(id) = cmd.args {
            debug!("Running CliCommand {:?} (empty args)", name);
            if let Err(e) = self.run_system_with_input(id, vec![]) {
                error!("CliCommand {:?} failed to run: {}", name, e);
            }
        } else {
            panic!("Missing CliCommand system registration");
        }
    }
}

impl CliCommandsRunExt for App {
    fn run_cli(&mut self, cli: &str) {
        self.world_mut().run_cli(cli);
    }
}

impl CliCommandsRunExt for Commands<'_, '_> {
    fn run_cli(&mut self, cli: &str) {
        self.queue(CliRunCommand(cli.to_owned()));
    }
}

pub struct CliRunCommand(pub String);

impl bevy::ecs::world::Command for CliRunCommand {
    fn apply(self, world: &mut World) {
        world.run_cli(&self.0);
    }
}

impl CliCommands {
    pub fn iter_names(&self) -> impl Iterator<Item = &str> {
        self.commands.keys().map(|s| s.as_str())
    }
    pub fn command_available(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }
    pub fn rename_command(&mut self, old_name: &str, new_name: &str) -> Result<(), ()> {
        if let Some(cmd) = self.commands.remove(old_name) {
            self.commands.insert(new_name.to_owned(), cmd);
            Ok(())
        } else {
            Err(())
        }
    }
}
