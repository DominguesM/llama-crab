// Master header included by bindgen.
//
// The `common` feature pulls in additional headers exposing our C++ helpers
// (chat template, OpenAI-compat parsing, JSON-schema-to-grammar, etc.).

#pragma once

#include <llama.h>
#include <ggml.h>
#include <ggml-backend.h>
#include <gguf.h>

#if defined(LLAMA_CRAB_HAS_COMMON)
#  include "wrappers/grammar.h"
#  include "wrappers/oaicompat.h"
#endif

#if defined(LLAMA_CRAB_HAS_MTMD)
#  include <mtmd.h>
#  include <mtmd-helper.h>
#  include "wrappers/mtmd_helpers.h"
#endif

#if defined(LLAMA_CRAB_HAS_LLGUIDANCE)
#  include "wrappers/llguidance_vtable.h"
#endif
