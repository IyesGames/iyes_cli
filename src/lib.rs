use bevy::prelude::*;
use bevy::ecs::system::BoxedSystem;
use bevy::utils::HashMap;

#[derive(Resource, Default)]
pub struct CliCommands {
    commands: HashMap<String, Option<BoxedSystem>>,
}

pub trait CliCommandsExt {
    fn register_clicommand<S, Param>(&mut self, name: &str, system: S)
        where S: IntoSystem<(), (), Param>;
    fn deregister_clicommand(&mut self, name: &str);
    fn run_clicommand(&mut self, name: &str);
}

impl CliCommandsExt for World {
    fn register_clicommand<S, Param>(&mut self, name: &str, system: S)
    where S: IntoSystem<(), (), Param> {
        self.deregister_clicommand(name);
        self.init_resource::<CliCommands>();
        let mut system = IntoSystem::into_system(system);
        system.initialize(self);
        self.resource_mut::<CliCommands>().commands
            .insert(name.to_owned(), Some(Box::new(system)));
        debug!("Registered CliCommand {:?}", name);
    }
    fn deregister_clicommand(&mut self, name: &str) {
        let Some(mut clicommands) = self.get_resource_mut::<CliCommands>() else {
            return;
        };
        if clicommands.commands.remove(name).is_some() {
            debug!("De-Registered CliCommand {:?}", name);
        }
    }
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

pub struct CliRunCommand(pub String);

impl bevy::ecs::system::Command for CliRunCommand {
    fn write(self, world: &mut World) {
        world.run_clicommand(&self.0);
    }
}
