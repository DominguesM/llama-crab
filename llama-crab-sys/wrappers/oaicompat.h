// C-ABI bridge for `common::chat::*` (chat templates + OAI-compat).
// Thin shim exposing `llama_rs_apply_chat_template_oaicompat` etc.

#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Status codes mirroring `llama_rs_status`.
typedef enum {
    LLAMA_RS_OK = 0,
    LLAMA_RS_INVALID_ARGUMENT = 1,
    LLAMA_RS_ALLOCATION_FAILED = 2,
    LLAMA_RS_EXCEPTION = 3,
} llama_rs_status;

// v0.1 stub — full implementation in v0.2.
llama_rs_status llama_rs_apply_chat_template_oaicompat(
    const char * tmpl,
    const char * messages_json,
    const char * tools_json,
    char ** out_prompt,
    char ** out_grammar
);

#ifdef __cplusplus
}
#endif
