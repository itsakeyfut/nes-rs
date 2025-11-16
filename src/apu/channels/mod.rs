//! APU channel implementations

pub mod noise;
pub mod pulse;
pub mod triangle;

pub use noise::NoiseChannel;
pub use pulse::PulseChannel;
pub use triangle::TriangleChannel;
