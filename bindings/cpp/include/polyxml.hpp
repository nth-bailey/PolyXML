#pragma once

#include <cstdint>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>

#include "polyxml.h"

namespace polyxml {

class Exception : public std::runtime_error {
public:
    explicit Exception(const std::string& msg) : std::runtime_error(msg) {}
};

class Value {
public:
    Value(polyxml_value_t* raw, bool owns) : raw_(raw), owns_(owns) {}

    ~Value() {
        if (owns_ && raw_) {
            polyxml_value_free(raw_);
        }
    }

    Value(const Value&) = delete;
    Value& operator=(const Value&) = delete;

    Value(Value&& other) noexcept : raw_(other.raw_), owns_(other.owns_) {
        other.raw_ = nullptr;
        other.owns_ = false;
    }

    Value& operator=(Value&& other) noexcept {
        if (this != &other) {
            if (owns_ && raw_) {
                polyxml_value_free(raw_);
            }
            raw_ = other.raw_;
            owns_ = other.owns_;
            other.raw_ = nullptr;
            other.owns_ = false;
        }
        return *this;
    }

    [[nodiscard]] bool is_null() const {
        return polyxml_value_is_null(raw_);
    }

    [[nodiscard]] std::optional<int64_t> as_int() const {
        int64_t v = 0;
        if (polyxml_value_get_int(raw_, &v) == POLYXML_OK) {
            return v;
        }
        return std::nullopt;
    }

    [[nodiscard]] std::optional<double> as_float() const {
        double v = 0.0;
        if (polyxml_value_get_float(raw_, &v) == POLYXML_OK) {
            return v;
        }
        return std::nullopt;
    }

    [[nodiscard]] std::optional<bool> as_bool() const {
        bool v = false;
        if (polyxml_value_get_bool(raw_, &v) == POLYXML_OK) {
            return v;
        }
        return std::nullopt;
    }

    [[nodiscard]] std::optional<std::string_view> as_string() const {
        const char* str = nullptr;
        size_t len = 0;
        if (polyxml_value_get_string(raw_, &str, &len) == POLYXML_OK && str != nullptr) {
            return std::string_view(str, len);
        }
        return std::nullopt;
    }

    [[nodiscard]] std::optional<Value> get(const std::string& key) const {
        const polyxml_value_t* field = polyxml_value_get_field(raw_, key.c_str());
        if (field) {
            return Value(const_cast<polyxml_value_t*>(field), false);
        }
        return std::nullopt;
    }

    [[nodiscard]] const polyxml_value_t* raw() const { return raw_; }

private:
    polyxml_value_t* raw_ = nullptr;
    bool owns_ = false;
};

class Schema {
public:
    explicit Schema(polyxml_schema_t* raw) : raw_(raw, polyxml_schema_free) {}

    [[nodiscard]] const polyxml_schema_t* raw() const { return raw_.get(); }

private:
    std::shared_ptr<polyxml_schema_t> raw_;
};

class SchemaBuilder {
public:
    explicit SchemaBuilder(const std::string& name)
        : raw_(polyxml_schema_builder_create(name.c_str())) {
        if (!raw_) {
            throw Exception("Failed to create schema builder");
        }
    }

    SchemaBuilder& add_attribute(const std::string& name, const std::string& xml_name, polyxml_scalar_type_t scalar_type) {
        polyxml_schema_builder_add_field(raw_, name.c_str(), xml_name.c_str(), POLYXML_FIELD_ATTRIBUTE, scalar_type);
        return *this;
    }

    SchemaBuilder& add_element(const std::string& name, const std::string& xml_name, polyxml_scalar_type_t scalar_type) {
        polyxml_schema_builder_add_field(raw_, name.c_str(), xml_name.c_str(), POLYXML_FIELD_ELEMENT, scalar_type);
        return *this;
    }

    Schema build() {
        polyxml_schema_t* schema = polyxml_schema_builder_build(raw_);
        raw_ = nullptr;
        if (!schema) {
            throw Exception("Failed to build schema");
        }
        return Schema(schema);
    }

private:
    polyxml_schema_builder_t* raw_ = nullptr;
};

inline Value deserialize(std::string_view xml, const Schema& schema) {
    polyxml_value_t* out_val = nullptr;
    auto code = polyxml_deserialize(
        reinterpret_cast<const uint8_t*>(xml.data()),
        xml.size(),
        schema.raw(),
        &out_val
    );

    if (code != POLYXML_OK || !out_val) {
        throw Exception("Deserialization error (code: " + std::to_string(code) + ")");
    }
    return Value(out_val, true);
}

inline std::string serialize(std::string_view root_name, const Value& val, const Schema& schema, int indent = 0) {
    uint8_t* out_bytes = nullptr;
    size_t out_len = 0;
    std::string root_str(root_name);

    auto code = polyxml_serialize(
        root_str.c_str(),
        val.raw(),
        schema.raw(),
        indent,
        &out_bytes,
        &out_len
    );

    if (code != POLYXML_OK || !out_bytes) {
        throw Exception("Serialization error (code: " + std::to_string(code) + ")");
    }

    std::string result(reinterpret_cast<char*>(out_bytes), out_len);
    polyxml_bytes_free(out_bytes, out_len);
    return result;
}

} // namespace polyxml
