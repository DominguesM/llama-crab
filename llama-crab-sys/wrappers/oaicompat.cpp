// Stub implementation. Real OAI-compat logic lands in v0.2 alongside
// the full chat-template engine.

#include "oaicompat.h"

#include <cstdlib>
#include <cstring>
#include <string>

extern "C" llama_rs_status llama_rs_apply_chat_template_oaicompat(
    const char * tmpl,
    const char * messages_json,
    const char * tools_json,
    char ** out_prompt,
    char ** out_grammar
) {
    (void)tmpl;
    (void)tools_json;
    if (messages_json == nullptr || out_prompt == nullptr) {
        return LLAMA_RS_INVALID_ARGUMENT;
    }
    // For now, just echo the messages JSON as the "prompt" and leave
    // grammar unset. The high-level Rust API can layer real template
    // rendering on top.
    std::string copy = std::string(messages_json);
    char * buf = static_cast<char *>(std::malloc(copy.size() + 1));
    if (!buf) {
        return LLAMA_RS_ALLOCATION_FAILED;
    }
    std::memcpy(buf, copy.data(), copy.size());
    buf[copy.size()] = '\0';
    *out_prompt = buf;
    if (out_grammar) {
        *out_grammar = nullptr;
    }
    return LLAMA_RS_OK;
}
