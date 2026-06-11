// `llama_crab_sys_wrappers` — small C++ shim layer that we always link into
// the FFI crate. Functions are conditionally registered; the entry points
// that are no-ops in v0.1 will be expanded in subsequent minor releases.

#include "grammar.h"
#include "oaicompat.h"
#include "mtmd_helpers.h"
#include "llguidance_vtable.h"
