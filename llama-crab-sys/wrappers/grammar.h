// Functional C-ABI bridge for `common::json_schema_to_grammar`.
// Compiled only when the `common` feature is enabled (LLAMA_BUILD_COMMON
// would be set, but we expose a C-ABI shim that compiles `common`'s
// JSON-schema-to-grammar.cpp directly so we don't have to link the full
// common library).

#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Convert a JSON Schema (string) into a GBNF grammar (string).
// Returns 0 on success and writes the grammar to `*out_grammar`.
// The caller must `free(*out_grammar)` when done.
//
// `force_gbnf` (when non-zero) keeps the grammar in the canonical
// GBNF form even when the input is simple enough to be expressed
// without backtracking.
int32_t llama_rs_json_schema_to_grammar(
    const char * json_schema,
    int32_t force_gbnf,
    char ** out_grammar
);

#ifdef __cplusplus
}
#endif
