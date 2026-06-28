#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <stdbool.h>
#include <stdint.h>

#define YAN_MAX_KEY_LEN 256
#define YAN_MAX_VALUE_LEN 4096
#define YAN_MAX_DEPTH 64
#define YAN_INITIAL_CAPACITY 16

typedef enum {
    YAN_NULL,
    YAN_BOOL,
    YAN_INT,
    YAN_FLOAT,
    YAN_STRING,
    YAN_ARRAY,
    YAN_OBJECT,
    YAN_TYPE_HINT
} yan_type_t;

typedef struct yan_value yan_value_t;
typedef struct yan_pair yan_pair_t;
typedef struct yan_array yan_array_t;
typedef struct yan_object yan_object_t;

struct yan_value {
    yan_type_t type;
    union {
        bool boolean;
        long long integer;
        double floating;
        char *string;
        yan_array_t *array;
        yan_object_t *object;
        struct {
            char *type_name;
            yan_value_t *value;
        } hint;
    } data;
};

struct yan_pair {
    char *key;
    yan_value_t *value;
};

struct yan_array {
    yan_value_t **items;
    size_t count;
    size_t capacity;
};

struct yan_object {
    yan_pair_t *pairs;
    size_t count;
    size_t capacity;
};

typedef struct {
    char *message;
    int line;
} yan_error_t;

typedef struct {
    char *content;
    int line;
    int indent;
} yan_line_t;

// ==================== MEMORY MANAGEMENT ====================

static void* yan_malloc(size_t size) {
    void *ptr = malloc(size);
    if (!ptr) {
        fprintf(stderr, "YAN: Out of memory\n");
        exit(1);
    }
    return ptr;
}

static void* yan_realloc(void *ptr, size_t size) {
    void *new_ptr = realloc(ptr, size);
    if (!new_ptr) {
        fprintf(stderr, "YAN: Out of memory\n");
        exit(1);
    }
    return new_ptr;
}

static char* yan_strdup(const char *str) {
    if (!str) return NULL;
    size_t len = strlen(str);
    char *copy = yan_malloc(len + 1);
    memcpy(copy, str, len + 1);
    return copy;
}

// ==================== VALUE CREATION ====================

static yan_value_t* yan_value_null(void) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_NULL;
    return v;
}

static yan_value_t* yan_value_bool(bool b) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_BOOL;
    v->data.boolean = b;
    return v;
}

static yan_value_t* yan_value_int(long long n) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_INT;
    v->data.integer = n;
    return v;
}

static yan_value_t* yan_value_float(double n) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_FLOAT;
    v->data.floating = n;
    return v;
}

static yan_value_t* yan_value_string(const char *s) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_STRING;
    v->data.string = yan_strdup(s);
    return v;
}

static yan_value_t* yan_value_type_hint(const char *type_name, yan_value_t *value) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_TYPE_HINT;
    v->data.hint.type_name = yan_strdup(type_name);
    v->data.hint.value = value;
    return v;
}

static yan_array_t* yan_array_new(void) {
    yan_array_t *arr = yan_malloc(sizeof(yan_array_t));
    arr->capacity = YAN_INITIAL_CAPACITY;
    arr->count = 0;
    arr->items = yan_malloc(sizeof(yan_value_t*) * arr->capacity);
    return arr;
}

static void yan_array_push(yan_array_t *arr, yan_value_t *value) {
    if (arr->count >= arr->capacity) {
        arr->capacity *= 2;
        arr->items = yan_realloc(arr->items, sizeof(yan_value_t*) * arr->capacity);
    }
    arr->items[arr->count++] = value;
}

static yan_value_t* yan_value_array(yan_array_t *arr) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_ARRAY;
    v->data.array = arr;
    return v;
}

static yan_object_t* yan_object_new(void) {
    yan_object_t *obj = yan_malloc(sizeof(yan_object_t));
    obj->capacity = YAN_INITIAL_CAPACITY;
    obj->count = 0;
    obj->pairs = yan_malloc(sizeof(yan_pair_t) * obj->capacity);
    return obj;
}

static void yan_object_set(yan_object_t *obj, const char *key, yan_value_t *value) {
    for (size_t i = 0; i < obj->count; i++) {
        if (strcmp(obj->pairs[i].key, key) == 0) {
            obj->pairs[i].value = value;
            return;
        }
    }
    if (obj->count >= obj->capacity) {
        obj->capacity *= 2;
        obj->pairs = yan_realloc(obj->pairs, sizeof(yan_pair_t) * obj->capacity);
    }
    obj->pairs[obj->count].key = yan_strdup(key);
    obj->pairs[obj->count].value = value;
    obj->count++;
}

static yan_value_t* yan_value_object(yan_object_t *obj) {
    yan_value_t *v = yan_malloc(sizeof(yan_value_t));
    v->type = YAN_OBJECT;
    v->data.object = obj;
    return v;
}

// ==================== PREPROCESSING ====================

static char* yan_preprocess(const char *source) {
    size_t len = strlen(source);
    char *result = yan_malloc(len * 2 + 1);
    size_t j = 0;

    for (size_t i = 0; i < len; i++) {
        if (source[i] == '\r') {
            if (i + 1 < len && source[i + 1] == '\n') {
                result[j++] = '\n';
                i++;
            } else {
                result[j++] = '\n';
            }
            continue;
        }

        if (source[i] == '/' && i + 1 < len && source[i + 1] == '*') {
            i += 2;
            while (i < len - 1 && !(source[i] == '*' && source[i + 1] == '/')) {
                i++;
            }
            i++;
            continue;
        }

        result[j++] = source[i];
    }
    result[j] = '\0';

    char *final = yan_malloc(strlen(result) + 1);
    size_t fj = 0;
    char *line = strtok(result, "\n");
    while (line) {
        bool in_quote = false;
        char quote_ch = 0;
        size_t hash_pos = SIZE_MAX;
        for (size_t k = 0; line[k]; k++) {
            char c = line[k];
            if (in_quote) {
                if (c == '\\' && line[k + 1]) { k++; continue; }
                if (c == quote_ch) in_quote = false;
            } else if (c == '"' || c == '\'') {
                in_quote = true;
                quote_ch = c;
            } else if (c == '#') {
                hash_pos = k;
                break;
            }
        }
        if (hash_pos != SIZE_MAX) line[hash_pos] = '\0';
        size_t llen = strlen(line);
        memcpy(final + fj, line, llen);
        fj += llen;
        final[fj++] = '\n';
        line = strtok(NULL, "\n");
    }
    final[fj] = '\0';

    free(result);
    return final;
}

// ==================== LINE SPLITTING ====================

static yan_line_t* yan_split_lines(const char *text, int *count) {
    int capacity = YAN_INITIAL_CAPACITY;
    yan_line_t *lines = yan_malloc(sizeof(yan_line_t) * capacity);
    int line_num = 1;
    const char *p = text;

    while (*p) {
        while (*p && *p == '\n') p++;
        if (!*p) break;

        const char *start = p;
        while (*p && *p != '\n') p++;

        size_t len = p - start;
        if (len == 0) continue;

        int indent = 0;
        while (indent < (int)len && (start[indent] == ' ' || start[indent] == '\t')) {
            if (start[indent] == '\t') indent += 2;
            else indent++;
        }

        while (len > 0 && isspace((unsigned char)start[len - 1])) len--;
        const char *content_start = start + indent;
        size_t content_len = len - indent;

        if (content_len == 0) continue;

        if (*count >= capacity) {
            capacity *= 2;
            lines = yan_realloc(lines, sizeof(yan_line_t) * capacity);
        }

        lines[*count].content = yan_malloc(content_len + 1);
        memcpy(lines[*count].content, content_start, content_len);
        lines[*count].content[content_len] = '\0';
        lines[*count].line = line_num;
        lines[*count].indent = indent;
        (*count)++;

        line_num++;
        if (*p == '\n') p++;
    }

    return lines;
}

// ==================== PARSING ====================

static yan_value_t* yan_parse_value(const char *raw, int line_num, yan_error_t **error);

static int yan_find_key_colon(const char *text) {
    bool in_quotes = false;
    for (int i = 0; text[i]; i++) {
        if (text[i] == '"' && !in_quotes) in_quotes = true;
        else if (text[i] == '"' && in_quotes) in_quotes = false;
        else if (text[i] == ':' && !in_quotes) return i;
    }
    return -1;
}

static char** yan_smart_split(const char *text, const char *delimiters, int *count) {
    int capacity = YAN_INITIAL_CAPACITY;
    char **parts = yan_malloc(sizeof(char*) * capacity);
    *count = 0;

    char current[YAN_MAX_VALUE_LEN] = {0};
    int ci = 0;
    bool in_quotes = false;
    int brace_depth = 0;

    for (int i = 0; text[i]; i++) {
        char ch = text[i];
        if (ch == '"' && !in_quotes) { in_quotes = true; current[ci++] = ch; }
        else if (ch == '"' && in_quotes) { in_quotes = false; current[ci++] = ch; }
        else if (ch == '{' && !in_quotes) { brace_depth++; current[ci++] = ch; }
        else if (ch == '}' && !in_quotes) { brace_depth--; current[ci++] = ch; }
        else if (strchr(delimiters, ch) && !in_quotes && brace_depth == 0) {
            current[ci] = '\0';
            if (*count >= capacity) {
                capacity *= 2;
                parts = yan_realloc(parts, sizeof(char*) * capacity);
            }
            parts[(*count)++] = yan_strdup(current);
            ci = 0;
            current[0] = '\0';
        } else {
            current[ci++] = ch;
        }
    }

    if (ci > 0) {
        current[ci] = '\0';
        if (*count >= capacity) {
            capacity *= 2;
            parts = yan_realloc(parts, sizeof(char*) * capacity);
        }
        parts[(*count)++] = yan_strdup(current);
    }

    return parts;
}

static bool yan_is_array(const char *text) {
    bool in_quotes = false;
    for (int i = 0; text[i]; i++) {
        if (text[i] == '"') in_quotes = !in_quotes;
        else if (text[i] == ';' && !in_quotes) return true;
    }
    return false;
}

static yan_object_t* yan_parse_inline_pairs(const char *text, yan_error_t **error);

static yan_value_t* yan_parse_value(const char *raw, int line_num, yan_error_t **error) {
    char value[YAN_MAX_VALUE_LEN];
    int vi = 0;
    for (int i = 0; raw[i] && vi < YAN_MAX_VALUE_LEN - 1; i++) {
        if (!isspace((unsigned char)raw[i]) || vi > 0) {
            value[vi++] = raw[i];
        }
    }
    value[vi] = '\0';

    while (vi > 0 && isspace((unsigned char)value[vi - 1])) {
        value[--vi] = '\0';
    }

    if (strlen(value) == 0) return yan_value_null();

    if (value[0] == '@') {
        char *space = strchr(value, ' ');
        char type_name[64] = {0};
        char raw_value[YAN_MAX_VALUE_LEN] = {0};

        if (space) {
            int tlen = space - value - 1;
            if (tlen > 0 && tlen < 63) {
                memcpy(type_name, value + 1, tlen);
                type_name[tlen] = '\0';
            }
            strcpy(raw_value, space + 1);
        } else {
            strncpy(type_name, value + 1, 63);
        }

        if (strcmp(type_name, "int") == 0) {
            return yan_value_int(atoll(raw_value));
        } else if (strcmp(type_name, "float") == 0) {
            return yan_value_float(atof(raw_value));
        } else if (strcmp(type_name, "bool") == 0) {
            char lower[16];
            for (int i = 0; raw_value[i] && i < 15; i++) lower[i] = tolower((unsigned char)raw_value[i]);
            lower[strlen(raw_value)] = '\0';
            return yan_value_bool(strcmp(lower, "true") == 0 || strcmp(lower, "yes") == 0 || strcmp(lower, "on") == 0);
        } else if (strcmp(type_name, "bigint") == 0) {
            const char *p = raw_value;
            if (*p == '-') p++;
            if (!*p) { fprintf(stderr, "Invalid @bigint value: \"%s\"\n", raw_value); return yan_value_null(); }
            for (; *p; p++) if (!isdigit((unsigned char)*p)) {
                fprintf(stderr, "Invalid @bigint value: \"%s\"\n", raw_value);
                return yan_value_null();
            }
            yan_value_t *inner = yan_value_string(raw_value);
            return yan_value_type_hint("bigint", inner);
        } else if (strcmp(type_name, "email") == 0) {
            const char *at = strchr(raw_value, '@');
            const char *dot = at ? strrchr(at, '.') : NULL;
            if (!at || at == raw_value || !dot || dot == at + 1 || !*(dot + 1)) {
                fprintf(stderr, "Invalid @email value: \"%s\"\n", raw_value);
                return yan_value_null();
            }
            yan_value_t *inner = yan_value_string(raw_value);
            return yan_value_type_hint("email", inner);
        } else if (strcmp(type_name, "ipv4") == 0) {
            int parts = 0, val = -1, ok = 1;
            char buf[YAN_MAX_VALUE_LEN];
            strcpy(buf, raw_value);
            char *tok = strtok(buf, ".");
            while (tok) {
                parts++;
                for (char *c = tok; *c; c++) if (!isdigit((unsigned char)*c)) ok = 0;
                val = atoi(tok);
                if (val < 0 || val > 255) ok = 0;
                tok = strtok(NULL, ".");
            }
            if (parts != 4 || !ok) {
                fprintf(stderr, "Invalid @ipv4 value: \"%s\"\n", raw_value);
                return yan_value_null();
            }
            yan_value_t *inner = yan_value_string(raw_value);
            return yan_value_type_hint("ipv4", inner);
        } else if (strcmp(type_name, "ipv6") == 0) {
            int has_colon = 0, ok = 1;
            for (const char *c = raw_value; *c; c++) {
                if (*c == ':') has_colon = 1;
                else if (!isxdigit((unsigned char)*c)) ok = 0;
            }
            if (!has_colon || !ok || raw_value[0] == '\0') {
                fprintf(stderr, "Invalid @ipv6 value: \"%s\"\n", raw_value);
                return yan_value_null();
            }
            yan_value_t *inner = yan_value_string(raw_value);
            return yan_value_type_hint("ipv6", inner);
        } else if (strcmp(type_name, "color") == 0) {
            char *cv = raw_value;
            size_t clen = strlen(cv);
            if (clen >= 2 && (cv[0] == '"' || cv[0] == '\'') && cv[clen - 1] == cv[0]) {
                cv[clen - 1] = '\0';
                cv++;
                clen -= 2;
            }
            int ok = (clen == 4 || clen == 7 || clen == 9) && cv[0] == '#';
            if (ok) for (size_t i = 1; i < clen; i++) if (!isxdigit((unsigned char)cv[i])) ok = 0;
            if (!ok) {
                fprintf(stderr, "Invalid @color value: \"%s\" (hex colors must be quoted, e.g. @color \"#ff0080\")\n", raw_value);
                return yan_value_null();
            }
            yan_value_t *inner = yan_value_string(cv);
            return yan_value_type_hint("color", inner);
        } else if (strcmp(type_name, "duration") == 0) {
            const char *p = raw_value;
            if (*p == '-') p++;
            int ok = *p != '\0';
            while (*p) {
                if (!isdigit((unsigned char)*p)) { ok = 0; break; }
                while (isdigit((unsigned char)*p)) p++;
                if (*p == '.') { p++; if (!isdigit((unsigned char)*p)) { ok = 0; break; } while (isdigit((unsigned char)*p)) p++; }
                if (*p == 'm' && *(p + 1) == 's') p += 2;
                else if (*p == 'd' || *p == 'h' || *p == 'm' || *p == 's') p++;
                else { ok = 0; break; }
            }
            if (!ok) {
                fprintf(stderr, "Invalid @duration value: \"%s\"\n", raw_value);
                return yan_value_null();
            }
            yan_value_t *inner = yan_value_string(raw_value);
            return yan_value_type_hint("duration", inner);
        } else {
            yan_value_t *inner = yan_value_string(raw_value);
            return yan_value_type_hint(type_name, inner);
        }
    }

    if (yan_is_array(value)) {
        int count;
        char **items = yan_smart_split(value, ";", &count);
        yan_array_t *arr = yan_array_new();
        for (int i = 0; i < count; i++) {
            yan_value_t *item = yan_parse_value(items[i], line_num, error);
            yan_array_push(arr, item);
            free(items[i]);
        }
        free(items);
        return yan_value_array(arr);
    }

    if (value[0] == '{') {
        int len = strlen(value);
        if (len > 2) {
            char inner[YAN_MAX_VALUE_LEN];
            memcpy(inner, value + 1, len - 2);
            inner[len - 2] = '\0';
            yan_object_t *obj = yan_parse_inline_pairs(inner, error);
            return yan_value_object(obj);
        }
        return yan_value_object(yan_object_new());
    }

    if (value[0] == '"') {
        int len = strlen(value);
        if (len < 2 || value[len - 1] != '"') {
            *error = yan_malloc(sizeof(yan_error_t));
            (*error)->message = yan_strdup("Unclosed string");
            (*error)->line = line_num;
            return yan_value_null();
        }
        char unquoted[YAN_MAX_VALUE_LEN];
        memcpy(unquoted, value + 1, len - 2);
        unquoted[len - 2] = '\0';
        return yan_value_string(unquoted);
    }

    char lower[16];
    for (int i = 0; value[i] && i < 15; i++) lower[i] = tolower((unsigned char)value[i]);
    lower[strlen(value)] = '\0';

    if (strcmp(lower, "true") == 0 || strcmp(lower, "yes") == 0 || strcmp(lower, "on") == 0) {
        return yan_value_bool(true);
    }
    if (strcmp(lower, "false") == 0 || strcmp(lower, "no") == 0 || strcmp(lower, "off") == 0) {
        return yan_value_bool(false);
    }
    if (strcmp(lower, "null") == 0 || strcmp(lower, "nil") == 0 || strcmp(lower, "_") == 0 || strcmp(lower, "~") == 0) {
        return yan_value_null();
    }

    bool is_float = false;
    bool is_number = true;
    int dot_count = 0, e_count = 0, digit_count = 0;
    for (int i = 0; value[i]; i++) {
        char c = value[i];
        if (c == '.') {
            dot_count++;
            is_float = true;
        } else if (c == 'e' || c == 'E') {
            e_count++;
            is_float = true;
        } else if (c == '-' || c == '+') {
            // only valid at start, or right after 'e'/'E'
            if (i != 0 && !(value[i-1] == 'e' || value[i-1] == 'E')) is_number = false;
        } else if (isdigit((unsigned char)c)) {
            digit_count++;
        } else {
            is_number = false;
        }
    }
    if (dot_count > 1 || e_count > 1 || digit_count == 0) is_number = false;

    if (is_number) {
        if (is_float) return yan_value_float(atof(value));
        return yan_value_int(atoll(value));
    }

    return yan_value_string(value);
}

static yan_object_t* yan_parse_inline_pairs(const char *text, yan_error_t **error) {
    yan_object_t *obj = yan_object_new();
    int count;
    char **pairs = yan_smart_split(text, ";,", &count);

    for (int i = 0; i < count; i++) {
        char *trimmed = pairs[i];
        while (isspace((unsigned char)*trimmed)) trimmed++;
        int tlen = strlen(trimmed);
        while (tlen > 0 && isspace((unsigned char)trimmed[tlen - 1])) trimmed[--tlen] = '\0';

        if (strlen(trimmed) == 0) continue;

        int colon = yan_find_key_colon(trimmed);
        if (colon == -1) continue;

        char key[YAN_MAX_KEY_LEN] = {0};
        memcpy(key, trimmed, colon);
        key[colon] = '\0';

        int kstart = 0, kend = colon - 1;
        while (kstart < kend && isspace((unsigned char)key[kstart])) kstart++;
        while (kend > kstart && isspace((unsigned char)key[kend])) key[kend--] = '\0';

        char *val = trimmed + colon + 1;
        while (isspace((unsigned char)*val)) val++;

        yan_value_t *parsed = yan_parse_value(val, 0, error);
        yan_object_set(obj, key + kstart, parsed);
        free(pairs[i]);
    }

    free(pairs);
    return obj;
}

// ==================== BLOCK PARSING ====================

static yan_object_t* yan_parse_block(yan_line_t *lines, int start, int *end, int base_indent, yan_error_t **error);

static yan_value_t* yan_parse_inline_object_lines(yan_line_t *lines, int start, int *end, const char *raw_value, yan_error_t **error) {
    int brace_count = 0;
    char content[YAN_MAX_VALUE_LEN * 4] = {0};
    int ci = 0;
    int i = start;

    while (i < *end) {
        const char *text = (i == start) ? raw_value : lines[i].content;
        int len = strlen(text);

        for (int k = 0; k < len; k++) {
            if (text[k] == '{') brace_count++;
            else if (text[k] == '}') {
                brace_count--;
                if (brace_count == 0) {
                    if (ci + k < sizeof(content) - 1) {
                        memcpy(content + ci, text, k);
                        content[ci + k] = '\0';
                    }
                    yan_object_t *obj = yan_parse_inline_pairs(content + 1, error);
                    *end = i + 1;
                    return yan_value_object(obj);
                }
            }
        }

        if (ci + len < sizeof(content) - 2) {
            memcpy(content + ci, text, len);
            ci += len;
            content[ci++] = ' ';
        }
        i++;
    }

    *error = yan_malloc(sizeof(yan_error_t));
    (*error)->message = yan_strdup("Unclosed '{'");
    (*error)->line = lines[start].line;
    return yan_value_null();
}

static yan_object_t* yan_parse_block(yan_line_t *lines, int start, int *end, int base_indent, yan_error_t **error) {
    yan_object_t *result = yan_object_new();
    int i = start;

    while (i < *end) {
        if (lines[i].indent <= base_indent) break;

        const char *content = lines[i].content;
        int colon = yan_find_key_colon(content);

        if (colon == -1) {
            *error = yan_malloc(sizeof(yan_error_t));
            char msg[256];
            snprintf(msg, sizeof(msg), "Expected ':' in '%s'", content);
            (*error)->message = yan_strdup(msg);
            (*error)->line = lines[i].line;
            return result;
        }

        char key[YAN_MAX_KEY_LEN] = {0};
        memcpy(key, content, colon);
        key[colon] = '\0';

        int kstart = 0;
        while (key[kstart] && isspace((unsigned char)key[kstart])) kstart++;
        int kend = strlen(key) - 1;
        while (kend > kstart && isspace((unsigned char)key[kend])) key[kend--] = '\0';

        char *raw_value = yan_malloc(strlen(content) - colon);
        strcpy(raw_value, content + colon + 1);
        while (isspace((unsigned char)*raw_value)) raw_value++;

        if (raw_value[0] == '{') {
            int next = *end;
            yan_value_t *val = yan_parse_inline_object_lines(lines, i, &next, raw_value, error);
            yan_object_set(result, key + kstart, val);
            i = next;
            continue;
        }

        if (i + 1 < *end && lines[i + 1].indent > lines[i].indent && strlen(raw_value) == 0) {
            int next = *end;
            yan_object_t *block = yan_parse_block(lines, i + 1, &next, lines[i].indent, error);
            yan_object_set(result, key + kstart, yan_value_object(block));
            i = next;
            continue;
        }

        yan_value_t *val = yan_parse_value(raw_value, lines[i].line, error);
        yan_object_set(result, key + kstart, val);
        i++;
    }

    *end = i;
    return result;
}

// ==================== PUBLIC API ====================

yan_object_t* yan_parse(const char *source, yan_error_t **error) {
    *error = NULL;
    char *cleaned = yan_preprocess(source);
    int count = 0;
    yan_line_t *lines = yan_split_lines(cleaned, &count);
    free(cleaned);

    int end = count;
    yan_object_t *result = yan_parse_block(lines, 0, &end, -1, error);

    for (int i = 0; i < count; i++) {
        free(lines[i].content);
    }
    free(lines);

    return result;
}

const char* yan_value_type_name(yan_type_t type) {
    switch (type) {
        case YAN_NULL: return "null";
        case YAN_BOOL: return "bool";
        case YAN_INT: return "int";
        case YAN_FLOAT: return "float";
        case YAN_STRING: return "string";
        case YAN_ARRAY: return "array";
        case YAN_OBJECT: return "object";
        case YAN_TYPE_HINT: return "type_hint";
        default: return "unknown";
    }
}

// ==================== PRINTING ====================

void yan_print_value(yan_value_t *value, int indent);

void yan_print_object(yan_object_t *obj, int indent) {
    printf("{\n");
    for (size_t i = 0; i < obj->count; i++) {
        printf("%*s  \"%s\": ", indent * 2, "", obj->pairs[i].key);
        yan_print_value(obj->pairs[i].value, indent + 1);
        if (i < obj->count - 1) printf(",");
        printf("\n");
    }
    printf("%*s}", indent * 2, "");
}

void yan_print_array(yan_array_t *arr, int indent) {
    printf("[\n");
    for (size_t i = 0; i < arr->count; i++) {
        printf("%*s  ", indent * 2, "");
        yan_print_value(arr->items[i], indent + 1);
        if (i < arr->count - 1) printf(",");
        printf("\n");
    }
    printf("%*s]", indent * 2, "");
}

void yan_print_value(yan_value_t *value, int indent) {
    if (!value) { printf("null"); return; }
    switch (value->type) {
        case YAN_NULL: printf("null"); break;
        case YAN_BOOL: printf(value->data.boolean ? "true" : "false"); break;
        case YAN_INT: printf("%lld", value->data.integer); break;
        case YAN_FLOAT: printf("%g", value->data.floating); break;
        case YAN_STRING: printf("\"%s\"", value->data.string); break;
        case YAN_ARRAY: yan_print_array(value->data.array, indent); break;
        case YAN_OBJECT: yan_print_object(value->data.object, indent); break;
        case YAN_TYPE_HINT: 
            printf("{\"__type\": \"%s\", \"__value\": ", value->data.hint.type_name);
            yan_print_value(value->data.hint.value, indent);
            printf("}");
            break;
    }
}

// ==================== MAIN ====================

int main(int argc, char **argv) {
    if (argc > 1) {
        // CLI mode: read .yan file from argv[1], print parsed JSON
        FILE *fp = fopen(argv[1], "rb");
        if (!fp) {
            fprintf(stderr, "Error: cannot open file '%s'\n", argv[1]);
            return 1;
        }

        fseek(fp, 0, SEEK_END);
        long fsize = ftell(fp);
        fseek(fp, 0, SEEK_SET);

        char *buf = malloc((size_t)fsize + 1);
        if (!buf) {
            fprintf(stderr, "Error: out of memory\n");
            fclose(fp);
            return 1;
        }
        size_t read_n = fread(buf, 1, (size_t)fsize, fp);
        buf[read_n] = '\0';
        fclose(fp);

        yan_error_t *error = NULL;
        yan_object_t *result = yan_parse(buf, &error);

        if (error) {
            fprintf(stderr, "Error at line %d: %s\n", error->line, error->message);
            free(buf);
            return 1;
        }

        yan_print_object(result, 0);
        printf("\n");
        free(buf);
        return 0;
    }

    // Default mode: built-in demo
    const char *test = 
        "# Server config\n"
        "server:\n"
        "  host: localhost\n"
        "  port: 8080\n"
        "  debug: off\n"
        "\n"
        "database:\n"
        "  driver: postgresql\n"
        "  pool: {min: 5; max: 20}\n"
        "\n"
        "tags: api; backend; v1\n";

    yan_error_t *error = NULL;
    yan_object_t *result = yan_parse(test, &error);

    if (error) {
        fprintf(stderr, "Error at line %d: %s\n", error->line, error->message);
        return 1;
    }

    printf("Parsed successfully!\n");
    printf("Root object has %zu keys:\n", result->count);
    for (size_t i = 0; i < result->count; i++) {
        printf("  %s -> %s\n", result->pairs[i].key, yan_value_type_name(result->pairs[i].value->type));
    }

    printf("\nFull output:\n");
    yan_print_object(result, 0);
    printf("\n");

    return 0;
}
