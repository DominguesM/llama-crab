// C-ABI bridge for `common::json_schema_to_grammar` (when `common` is built).

#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Convert a JSON Schema string to a GBNF grammar string.
// Caller frees the returned `out_grammar` with `free()`.
int llama_rs_json_schema_to_grammar(
    const char * json_schema,
    char ** out_grammar
);

#ifdef __cplusplus
}
#endif
