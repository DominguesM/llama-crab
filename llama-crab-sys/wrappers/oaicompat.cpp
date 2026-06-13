// Compatibility stub kept out of bindgen in v0.1.300.

#include "oaicompat.h"

extern "C" llama_rs_status llama_rs_apply_chat_template_oaicompat(
    const char * tmpl,
    const char * messages_json,
    const char * tools_json,
    char ** out_prompt,
    char ** out_grammar
) {
    (void)tmpl;
    (void)messages_json;
    (void)tools_json;
    if (out_prompt != nullptr) {
        *out_prompt = nullptr;
    }
    if (out_grammar != nullptr) {
        *out_grammar = nullptr;
    }
    return LLAMA_RS_UNSUPPORTED;
}
