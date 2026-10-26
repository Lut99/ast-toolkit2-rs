//  RENDERERS.rs
//    by Lut99
//
//  Description:
//!   Provides some standard library [`Renderer`](super::Renderer)s to use in
//!   most projects.
//

// Nested modules
mod hex;
mod text;

// Imports
pub use hex::HexRenderer;
pub use text::TextRenderer;
