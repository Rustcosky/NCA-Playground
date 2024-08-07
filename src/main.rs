use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 0.5)))
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            nca_playground::NCAPlaygroundPlugin,
        ))
        .run();
}
