use std::any::Any;
use std::io;
use std::path::Path;

use crate::buffer::Buffer;
use crate::EngineError;
use crate::scene::Scene;

pub trait SceneLoaderComponent: Any + Send + Sync {
    fn load(&self, mod_source: &Path, path: &str) -> Result<Scene, EngineError>;
}

pub trait CompositorComponent: Any + Send + Sync {
    fn compose(&self, scene: &Scene, buffer: &mut Buffer);
}

pub trait AnimatorComponent: Any + Send + Sync {
    fn tick(&mut self, scene: &mut Scene);
}

pub trait RendererComponent: Any + Send + Sync {
    fn flush(&mut self, buffer: &Buffer) -> io::Result<()>;
}
