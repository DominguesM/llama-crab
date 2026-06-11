//! Multimodal (vision + audio) support, gated behind the `mtmd` feature.

#[cfg(feature = "mtmd")]
mod inner {
    // The multimodal wrappers live in the `multimodal/` submodule.
    pub use crate::multimodal::*;
}
