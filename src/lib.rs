use godot::prelude::*;

struct PackExtension;
pub mod pack;

#[gdextension]
unsafe impl ExtensionLibrary for PackExtension {}
