// Stub JSON-schema → GBNF converter.
// Real implementation in v0.2: re-export `common::json_schema_to_grammar`
// once `LLAMA_BUILD_COMMON=ON` and the `common` library is linked.

#include "grammar.h"

#include <cstdlib>
#include <cstring>
#include <string>

extern "C" int llama_rs_json_schema_to_grammar(
    const char * json_schema,
    char ** out_grammar
) {
    if (json_schema == nullptr || out_grammar == nullptr) {
        return 1;
    }
    // Fall back to a permissive grammar that accepts any string.
    static const char * fallback = "root ::= .*";
    std::string copy = fallback;
    char * buf = static_cast<char *>(std::malloc(copy.size() + 1));
    if (!buf) {
        return 2;
    }
    std::memcpy(buf, copy.data(), copy.size() + 1);
    *out_grammar = buf;
    return 0;
}
