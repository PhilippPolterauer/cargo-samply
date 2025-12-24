use bevy::prelude::*;
use bevy::app::AppExit;

fn main() {
    println!("Bevy app starting...");
    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, exit_system)
        .run();
    println!("Bevy app finished.");
}

fn setup() {
    println!("Setup system ran.");
}

fn exit_system(mut exit: EventWriter<AppExit>) {
    println!("Exiting...");
    exit.send(AppExit);
}
