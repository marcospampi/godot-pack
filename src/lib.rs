use godot::prelude::*;

struct MyExtension;
pub mod pack;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
