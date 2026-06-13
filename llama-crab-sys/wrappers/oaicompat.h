// Reserved C-ABI bridge for `common::chat::*` (chat templates + OAI-compat).
// Not included by wrapper.h until it is backed by upstream implementation.

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
    LLAMA_RS_UNSUPPORTED = 4,
} llama_rs_status;

// v0.1.300 compatibility stub. Returns LLAMA_RS_UNSUPPORTED.
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
