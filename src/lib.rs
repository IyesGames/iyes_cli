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
    pub commands_args: HashMap<String, Option<Box<dyn System<In = Vec<String>, Out = ()>>>>,
    pub commands_noargs: HashMap<String, Option<Box<dyn System<In = (), Out = ()>>>>,
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
        S: IntoSystem<(), (), Param>;
    /// Create a new "console command" with the given string and system/implementation
    ///
    /// If a command with the same name already exists, it is replaced.
    fn register_clicommand_args<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<Vec<String>, (), Param>;
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
        S: IntoSystem<(), (), Param>,
    {
        self.deregister_clicommand(name, false);
        self.init_resource::<CliCommands>();
        let mut system = IntoSystem::into_system(system);
        system.initialize(self);
        self.resource_mut::<CliCommands>()
            .commands_noargs
            .insert(name.to_owned(), Some(Box::new(system)));
        self
    }
    fn register_clicommand_args<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<Vec<String>, (), Param>,
    {
        self.deregister_clicommand(name, true);
        self.init_resource::<CliCommands>();
        let mut system = IntoSystem::into_system(system);
        system.initialize(self);
        self.resource_mut::<CliCommands>()
            .commands_args
            .insert(name.to_owned(), Some(Box::new(system)));
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
        S: IntoSystem<(), (), Param>,
    {
        self.world.register_clicommand_noargs(name, system);
        self
    }
    fn register_clicommand_args<S, Param>(&mut self, name: &str, system: S) -> &mut Self
    where
        S: IntoSystem<Vec<String>, (), Param>,
    {
        self.world.register_clicommand_args(name, system);
        self
    }
    fn deregister_clicommand(&mut self, name: &str, has_args: bool) -> &mut Self {
        self.world.deregister_clicommand(name, has_args);
        self
    }
}

impl CliCommandsRunExt for World {
    fn run_clicommand(&mut self, cli: &str) {
        // TODO: support quotes and other such fancy syntax?
        let mut iter = cli.split_ascii_whitespace();
        let Some(name) = iter.next() else {
            warn!("Attempted to run empty CLI string!");
            return;
        };
        let args: Vec<String> = iter.map(|s| s.to_owned()).collect();

        if !args.is_empty() {
            let Some(mut system) = self
                .resource_mut::<CliCommands>()
                .bypass_change_detection()
                .commands_args
                .get_mut(name)
                .and_then(|opt| opt.take())
            else {
                warn!("CliCommand {:?} not found!", name);
                return;
            };
            debug!("Running CliCommand {:?}, args: {:?}", name, args);
            system.run(args, self);
            system.apply_deferred(self);
            if let Some(cmd) = self
                .resource_mut::<CliCommands>()
                .bypass_change_detection()
                .commands_args
                .get_mut(name)
            {
                *cmd = Some(system);
            }
        } else {
            let Some(mut system) = self
                .resource_mut::<CliCommands>()
                .bypass_change_detection()
                .commands_noargs
                .get_mut(name)
                .and_then(|opt| opt.take())
            else {
                warn!("CliCommand {:?} not found!", name);
                return;
            };
            debug!("Running CliCommand {:?} (noargs)", name);
            system.run((), self);
            system.apply_deferred(self);
            if let Some(cmd) = self
                .resource_mut::<CliCommands>()
                .bypass_change_detection()
                .commands_noargs
                .get_mut(name)
            {
                *cmd = Some(system);
            }
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
    fn apply(self, world: &mut World) {
        world.run_clicommand(&self.0);
    }
}
