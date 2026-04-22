pub mod cockpit;
pub mod cube_sphere;
pub mod poly_sphere;
pub mod terrain_plane;
pub mod terrain_sphere;
pub mod uv_sphere;

pub use cockpit::{simulator_cockpit, CockpitParams};
pub use cube_sphere::cube_sphere;
pub use poly_sphere::{icosa_sphere, octa_sphere, tetra_sphere};
pub use terrain_plane::{terrain_plane, TerrainParams};
pub use terrain_sphere::{earth_terrain_sphere, terrain_sphere};
pub use uv_sphere::uv_sphere;
