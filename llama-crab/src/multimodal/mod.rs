//! Multimodal (vision + audio) support via `mtmd`.
//!
//! This module wraps the `mtmd` C API exposed by llama.cpp's
//! `tools/mtmd/` directory. To use it, enable the `mtmd` cargo feature
//! and load a GGUF model that has a paired `mmproj` (multimodal projector)
//! file.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "mtmd")] {
//! use llama_crab::multimodal::{MtmdContext, MtmdContextParams, MtmdInputText, MtmdBitmap};
//! use llama_crab::{Llama, LlamaParams};
//!
//! let mut llama = Llama::load(
//!     LlamaParams::new("gemma-4-E4B-it-Q4_K_M.gguf").with_n_ctx(4096)
//! )?;
//! let mtmd = MtmdContext::init_from_file("gemma-4-E4B-it-mmproj.gguf", llama.model())?;
//! let bitmap = MtmdBitmap::from_file("image.png")?;
//! let chunks = mtmd.tokenize(MtmdInputText::new("Describe this image"), &[&bitmap])?;
//! // ... feed chunks into llama.context().decode() ...
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::path::Path;
use std::ptr::NonNull;

use llama_crab_sys as sys;

use crate::error::{LlamaError, Result};
use crate::model::LlamaModel;

/// Initialization parameters for an [`MtmdContext`].
#[derive(Debug, Clone)]
pub struct MtmdContextParams {
    /// Whether to use GPU for the projector model.
    pub use_gpu: bool,
    /// Print timing information on shutdown.
    pub print_timings: bool,
    /// Number of threads to use for the projector.
    pub n_threads: i32,
}

impl Default for MtmdContextParams {
    fn default() -> Self {
        Self {
            use_gpu: true,
            print_timings: false,
            n_threads: 1,
        }
    }
}

impl MtmdContextParams {
    /// Convert to the C struct.
    fn to_c(&self) -> sys::mtmd_context_params {
        let mut p = unsafe { sys::mtmd_context_params_default() };
        p.use_gpu = self.use_gpu;
        p.print_timings = self.print_timings;
        p.n_threads = self.n_threads;
        p
    }
}

/// An initialized multimodal context, bound to a text [`LlamaModel`].
#[derive(Debug)]
pub struct MtmdContext {
    handle: NonNull<sys::mtmd_context>,
}

impl MtmdContext {
    /// Initialize the multimodal context from an `mmproj` GGUF file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or the projector is
    /// incompatible with the supplied text model.
    pub fn init_from_file(
        mmproj_path: impl AsRef<Path>,
        text_model: &LlamaModel,
    ) -> Result<Self> {
        Self::init_from_file_with(mmproj_path, text_model, MtmdContextParams::default())
    }

    /// Initialize with custom parameters.
    pub fn init_from_file_with(
        mmproj_path: impl AsRef<Path>,
        text_model: &LlamaModel,
        params: MtmdContextParams,
    ) -> Result<Self> {
        let cpath = std::ffi::CString::new(mmproj_path.as_ref().display().to_string())?;
        let handle = unsafe {
            sys::mtmd_init_from_file(cpath.as_ptr(), text_model.raw(), params.to_c())
        };
        NonNull::new(handle)
            .map(|handle| Self { handle })
            .ok_or_else(|| {
                LlamaError::ModelLoad(format!(
                    "mtmd_init_from_file({}) failed",
                    mmproj_path.as_ref().display()
                ))
            })
    }

    /// Whether the projector requires non-causal attention.
    #[must_use]
    pub fn decode_use_non_causal(&self, chunk: &MtmdInputChunk) -> bool {
        unsafe { sys::mtmd_decode_use_non_causal(self.handle.as_ptr(), chunk.handle.as_ptr()) }
    }

    /// Whether the projector uses M-RoPE.
    #[must_use]
    pub fn decode_use_mrope(&self) -> bool {
        unsafe { sys::mtmd_decode_use_mrope(self.handle.as_ptr()) }
    }

    /// Whether the projector supports image inputs.
    #[must_use]
    pub fn support_vision(&self) -> bool {
        unsafe { sys::mtmd_support_vision(self.handle.as_ptr()) }
    }

    /// Whether the projector supports audio inputs.
    #[must_use]
    pub fn support_audio(&self) -> bool {
        unsafe { sys::mtmd_support_audio(self.handle.as_ptr()) }
    }

    /// Audio sample rate expected by the projector.
    #[must_use]
    pub fn audio_sample_rate(&self) -> i32 {
        unsafe { sys::mtmd_get_audio_sample_rate(self.handle.as_ptr()) }
    }

    /// Tokenize `text` along with the supplied bitmaps into a list of chunks.
    ///
    /// # Errors
    /// Returns an error on tokenization failure.
    pub fn tokenize(
        &self,
        text: MtmdInputText<'_>,
        bitmaps: &[&MtmdBitmap],
    ) -> Result<MtmdInputChunks> {
        let c_text = text.into_c();
        let mut bitmap_ptrs: Vec<*const sys::mtmd_bitmap> =
            bitmaps.iter().map(|b| b.handle.as_ptr().cast_const()).collect();
        let mut chunks = MtmdInputChunks::new()?;
        let rc = unsafe {
            sys::mtmd_tokenize(
                self.handle.as_ptr(),
                chunks.handle.as_ptr(),
                &c_text,
                bitmap_ptrs.as_mut_ptr(),
                bitmap_ptrs.len(),
            )
        };
        if rc != 0 {
            return Err(LlamaError::Batch(format!("mtmd_tokenize: {rc}")));
        }
        Ok(chunks)
    }
}

impl Drop for MtmdContext {
    fn drop(&mut self) {
        // Safety: `handle` is exclusively owned and was returned by
        // `mtmd_init_from_file`.
        unsafe { sys::mtmd_free(self.handle.as_ptr()) };
    }
}

// Safety: the mtmd context is not thread-safe per llama.cpp docs.
unsafe impl Send for MtmdContext {}

/// Text input paired with tokenization options for [`MtmdContext::tokenize`].
#[derive(Debug, Clone)]
pub struct MtmdInputText<'a> {
    /// Raw text (may contain `<image>`/`<audio>` markers).
    pub text: &'a str,
    /// Add special tokens (BOS, chat template tokens).
    pub add_special: bool,
    /// Parse special tokens like `<|image|>`.
    pub parse_special: bool,
}

impl<'a> MtmdInputText<'a> {
    /// Construct a simple text input.
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            add_special: true,
            parse_special: true,
        }
    }

    /// Build the C struct.
    fn into_c(self) -> sys::mtmd_input_text {
        // mtmd_input_text does NOT copy the text — the C struct holds a
        // pointer. We leak the CString to give it `'static` lifetime; the
        // caller will typically re-tokenize for each prompt, and the leak
        // is bounded.
        let cstr = std::ffi::CString::new(self.text).expect("text with no nul bytes");
        sys::mtmd_input_text {
            text: cstr.into_raw().cast_const(),
            add_special: self.add_special,
            parse_special: self.parse_special,
        }
    }
}

/// A single bitmap (image or audio waveform) to feed into an [`MtmdContext`].
#[derive(Debug)]
pub struct MtmdBitmap {
    handle: NonNull<sys::mtmd_bitmap>,
}

impl MtmdBitmap {
    /// Construct a bitmap from raw pixel data (`nx × ny × 3` bytes, RGB).
    ///
    /// # Errors
    /// Returns an error if the dimensions are invalid.
    pub fn from_image_data(nx: u32, ny: u32, data: &[u8]) -> Result<Self> {
        if data.len() < (nx as usize) * (ny as usize) * 3 {
            return Err(LlamaError::Batch("bitmap data too small".into()));
        }
        let handle = unsafe { sys::mtmd_bitmap_init(nx, ny, data.as_ptr()) };
        NonNull::new(handle)
            .map(|handle| Self { handle })
            .ok_or(LlamaError::Batch("mtmd_bitmap_init returned null".into()))
    }

    /// Construct a bitmap from a float audio buffer.
    pub fn from_audio_data(data: &[f32]) -> Result<Self> {
        let handle = unsafe {
            sys::mtmd_bitmap_init_from_audio(data.len(), data.as_ptr())
        };
        NonNull::new(handle)
            .map(|handle| Self { handle })
            .ok_or(LlamaError::Batch("mtmd_bitmap_init_from_audio returned null".into()))
    }

    /// Construct a bitmap from an image file on disk.
    ///
    /// Requires the `image` cargo feature. The image is decoded to RGB8
    /// before being passed to mtmd.
    #[cfg(feature = "mtmd")]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let img = image::open(path.as_ref())
            .map_err(|e| LlamaError::Batch(format!("image decode: {e}")))?;
        let rgb8 = img.to_rgb8();
        let (nx, ny) = (rgb8.width(), rgb8.height());
        let bytes = rgb8.into_raw();
        let handle = unsafe { sys::mtmd_bitmap_init(nx, ny, bytes.as_ptr()) };
        NonNull::new(handle)
            .map(|handle| Self { handle })
            .ok_or(LlamaError::Batch("mtmd_bitmap_init returned null".into()))
    }

    /// Width (px) of the image bitmap.
    #[must_use]
    pub fn nx(&self) -> u32 {
        unsafe { sys::mtmd_bitmap_get_nx(self.handle.as_ptr()) }
    }

    /// Height (px) of the image bitmap.
    #[must_use]
    pub fn ny(&self) -> u32 {
        unsafe { sys::mtmd_bitmap_get_ny(self.handle.as_ptr()) }
    }

    /// Number of bytes backing the bitmap.
    #[must_use]
    pub fn n_bytes(&self) -> usize {
        unsafe { sys::mtmd_bitmap_get_n_bytes(self.handle.as_ptr()) }
    }

    /// Whether the bitmap carries audio (as opposed to image) data.
    #[must_use]
    pub fn is_audio(&self) -> bool {
        unsafe { sys::mtmd_bitmap_is_audio(self.handle.as_ptr()) }
    }
}

impl Drop for MtmdBitmap {
    fn drop(&mut self) {
        // Safety: `handle` is exclusively owned.
        unsafe { sys::mtmd_bitmap_free(self.handle.as_ptr()) };
    }
}

/// A list of tokenized chunks produced by [`MtmdContext::tokenize`].
#[derive(Debug)]
pub struct MtmdInputChunks {
    handle: NonNull<sys::mtmd_input_chunks>,
}

impl MtmdInputChunks {
    fn new() -> Result<Self> {
        let handle = unsafe { sys::mtmd_input_chunks_init() };
        NonNull::new(handle)
            .map(|handle| Self { handle })
            .ok_or(LlamaError::Batch("mtmd_input_chunks_init returned null".into()))
    }

    /// Number of chunks in the list.
    #[must_use]
    pub fn len(&self) -> usize {
        unsafe { sys::mtmd_input_chunks_size(self.handle.as_ptr()) }
    }

    /// True if there are no chunks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the chunk at `idx` (a borrowed view; lives as long as `self`).
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<MtmdInputChunk<'_>> {
        let p = unsafe { sys::mtmd_input_chunks_get(self.handle.as_ptr(), idx) };
        NonNull::new(p.cast_mut()).map(|handle| MtmdInputChunk {
            handle,
            _owned: false,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Evaluate all chunks: encode the images, then decode the resulting
    /// tokens. The number of new positions consumed is written to
    /// `new_n_past`.
    ///
    /// # Safety
    /// `llama_ctx` must be a live, unaliased `llama_context`.
    pub unsafe fn eval(
        &self,
        mtmd_ctx: &MtmdContext,
        llama_ctx: *mut sys::llama_context,
        n_past: i32,
        seq_id: i32,
        n_batch: i32,
        logits_last: bool,
    ) -> Result<i32> {
        let mut new_n_past: i32 = 0;
        let rc = unsafe {
            sys::mtmd_helper_eval_chunks(
                mtmd_ctx.handle.as_ptr(),
                llama_ctx,
                self.handle.as_ptr(),
                n_past,
                seq_id,
                n_batch,
                logits_last,
                &mut new_n_past,
            )
        };
        if rc != 0 {
            return Err(LlamaError::Ffi(rc));
        }
        Ok(new_n_past)
    }
}

impl Drop for MtmdInputChunks {
    fn drop(&mut self) {
        // Safety: `handle` is exclusively owned.
        unsafe { sys::mtmd_input_chunks_free(self.handle.as_ptr()) };
    }
}

/// A single chunk (text or image embedding). Lifetime-bound to the
/// [`MtmdInputChunks`] that produced it.
#[derive(Debug)]
pub struct MtmdInputChunk<'a> {
    handle: NonNull<sys::mtmd_input_chunk>,
    _owned: bool,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> MtmdInputChunk<'a> {
    /// Number of tokens the chunk decodes into.
    #[must_use]
    pub fn n_tokens(&self) -> usize {
        unsafe { sys::mtmd_input_chunk_get_n_tokens(self.handle.as_ptr()) }
    }

    /// Number of positions the chunk consumes in the KV cache.
    #[must_use]
    pub fn n_pos(&self) -> i32 {
        unsafe { sys::mtmd_input_chunk_get_n_pos(self.handle.as_ptr()) }
    }
}
