// Compile-time-optional bridge. If `common` is enabled we link against the
// full llama.cpp common library; otherwise we provide a pure Rust fallback
// (declarative JSON → GBNF).
//
// This file uses the upstream `common::json_schema_to_grammar` only when
// the macro `LLAMA_CRAB_HAS_COMMON_SCHEMA_LIB` is defined. The Rust side
// detects at build time which path is available.

#include "grammar.h"

#include <cstdlib>
#include <cstring>
#include <string>

#if defined(LLAMA_CRAB_HAS_COMMON_SCHEMA_LIB)
#  include "nlohmann/json.hpp"
#  include "common/json-schema-to-grammar.h"

extern "C" int32_t llama_rs_json_schema_to_grammar(
    const char * json_schema,
    int32_t force_gbnf,
    char ** out_grammar
) {
    if (json_schema == nullptr || out_grammar == nullptr) {
        return 1;
    }
    try {
        auto schema = nlohmann::ordered_json::parse(json_schema);
        std::string gbnf = common::json_schema_to_grammar(
            schema, static_cast<bool>(force_gbnf)
        );
        char * buf = static_cast<char *>(std::malloc(gbnf.size() + 1));
        if (!buf) {
            return 2;
        }
        std::memcpy(buf, gbnf.data(), gbnf.size());
        buf[gbnf.size()] = '\0';
        *out_grammar = buf;
        return 0;
    } catch (const std::exception & e) {
        return 3;
    } catch (...) {
        return 4;
    }
}
#else
// Fallback: a permissive grammar that accepts any UTF-8 string. The Rust
// side has a richer JSON-Schema converter that gets used in this path.
extern "C" int32_t llama_rs_json_schema_to_grammar(
    const char * json_schema,
    int32_t force_gbnf,
    char ** out_grammar
) {
    (void)json_schema;
    (void)force_gbnf;
    if (out_grammar == nullptr) {
        return 1;
    }
    static const char * kFallback =
        "root ::= \"{\"\n"
        " | string\n"
        " | number\n"
        " | boolean\n"
        " | null\n"
        " | array\n"
        " | object\n";
    const size_t n = std::strlen(kFallback);
    char * buf = static_cast<char *>(std::malloc(n + 1));
    if (!buf) {
        return 2;
    }
    std::memcpy(buf, kFallback, n + 1);
    *out_grammar = buf;
    return 0;
}
#endif
