package io.polyxml;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

/**
 * PolyXML: High-performance, polyglot XML data-binding engine for Java.
 * Powered by Java 22+ Foreign Function &amp; Memory API (Project Panama).
 */
public final class PolyXML {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup LOOKUP;

    static {
        System.loadLibrary("polyxml");
        LOOKUP = SymbolLookup.loaderLookup();
    }

    public enum FieldKind {
        ATTRIBUTE(0),
        ELEMENT(1),
        TEXT(2);

        final int code;
        FieldKind(int code) { this.code = code; }
    }

    public enum ScalarType {
        STRING(0),
        INT(1),
        FLOAT(2),
        BOOL(3),
        DECIMAL(4),
        XML_DATE(5),
        XML_DATETIME(6),
        ANY(7);

        final int code;
        ScalarType(int code) { this.code = code; }
    }

    public static final class Schema implements AutoCloseable {
        final MemorySegment handle;

        Schema(MemorySegment handle) {
            this.handle = handle;
        }

        public MemorySegment handle() {
            return handle;
        }

        @Override
        public void close() {
            try {
                MethodHandle freeHandle = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_schema_free").orElseThrow(),
                    FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
                );
                freeHandle.invokeExact(handle);
            } catch (Throwable t) {
                throw new RuntimeException("Failed to free native schema", t);
            }
        }
    }

    public static final class SchemaBuilder {
        private final MemorySegment handle;

        public SchemaBuilder(String name) {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment cName = arena.allocateFrom(name);
                MethodHandle createHandle = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_schema_builder_create").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                this.handle = (MemorySegment) createHandle.invokeExact(cName);
            } catch (Throwable t) {
                throw new RuntimeException("Failed to create schema builder", t);
            }
        }

        public SchemaBuilder setNamespace(String namespaceURI) {
            if (namespaceURI == null || namespaceURI.isEmpty()) {
                return this;
            }
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment cNs = arena.allocateFrom(namespaceURI);
                MethodHandle setNsHandle = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_schema_builder_set_namespace").orElseThrow(),
                    FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                setNsHandle.invokeExact(handle, cNs);
            } catch (Throwable t) {
                throw new RuntimeException("Failed to set namespace on schema", t);
            }
            return this;
        }

        public SchemaBuilder addField(String name, String xmlName, FieldKind kind, ScalarType scalar) {
            return addField(name, xmlName, kind, scalar, null);
        }

        public SchemaBuilder addField(String name, String xmlName, FieldKind kind, ScalarType scalar, String namespaceURI) {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment cName = arena.allocateFrom(name);
                MemorySegment cXmlName = arena.allocateFrom(xmlName);
                MemorySegment cNs = (namespaceURI != null && !namespaceURI.isEmpty())
                    ? arena.allocateFrom(namespaceURI)
                    : MemorySegment.NULL;

                MethodHandle addHandle = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_schema_builder_add_field_with_namespace").orElseThrow(),
                    FunctionDescriptor.ofVoid(
                        ValueLayout.ADDRESS,
                        ValueLayout.ADDRESS,
                        ValueLayout.ADDRESS,
                        ValueLayout.JAVA_INT,
                        ValueLayout.JAVA_INT,
                        ValueLayout.ADDRESS
                    )
                );
                addHandle.invokeExact(handle, cName, cXmlName, kind.code, scalar.code, cNs);
            } catch (Throwable t) {
                throw new RuntimeException("Failed to add field to schema", t);
            }
            return this;
        }

        public Schema build() {
            try {
                MethodHandle buildHandle = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_schema_builder_build").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                MemorySegment schemaSeg = (MemorySegment) buildHandle.invokeExact(handle);
                return new Schema(schemaSeg);
            } catch (Throwable t) {
                throw new RuntimeException("Failed to build schema", t);
            }
        }
    }

    public static final class Value implements AutoCloseable {
        final MemorySegment handle;
        final boolean owns;

        Value(MemorySegment handle, boolean owns) {
            this.handle = handle;
            this.owns = owns;
        }

        public MemorySegment handle() {
            return handle;
        }

        public boolean isNull() {
            if (handle == null || handle.equals(MemorySegment.NULL)) {
                return true;
            }
            try {
                MethodHandle isNullHandle = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_value_is_null").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN, ValueLayout.ADDRESS)
                );
                return (boolean) isNullHandle.invokeExact(handle);
            } catch (Throwable t) {
                return true;
            }
        }

        public Optional<Long> getInt() {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment outInt = arena.allocate(ValueLayout.JAVA_LONG);
                MethodHandle handleFn = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_value_get_int").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                int code = (int) handleFn.invokeExact(handle, outInt);
                if (code == 0) {
                    return Optional.of(outInt.get(ValueLayout.JAVA_LONG, 0));
                }
                return Optional.empty();
            } catch (Throwable t) {
                throw new RuntimeException("Failed to get int from value", t);
            }
        }

        public Optional<Double> getFloat() {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment outFloat = arena.allocate(ValueLayout.JAVA_DOUBLE);
                MethodHandle handleFn = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_value_get_float").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                int code = (int) handleFn.invokeExact(handle, outFloat);
                if (code == 0) {
                    return Optional.of(outFloat.get(ValueLayout.JAVA_DOUBLE, 0));
                }
                return Optional.empty();
            } catch (Throwable t) {
                throw new RuntimeException("Failed to get float from value", t);
            }
        }

        public Optional<Boolean> getBool() {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment outBool = arena.allocate(ValueLayout.JAVA_BOOLEAN);
                MethodHandle handleFn = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_value_get_bool").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                int code = (int) handleFn.invokeExact(handle, outBool);
                if (code == 0) {
                    return Optional.of(outBool.get(ValueLayout.JAVA_BOOLEAN, 0));
                }
                return Optional.empty();
            } catch (Throwable t) {
                throw new RuntimeException("Failed to get bool from value", t);
            }
        }

        public Optional<String> getString() {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment outStr = arena.allocate(ValueLayout.ADDRESS);
                MemorySegment outLen = arena.allocate(ValueLayout.JAVA_LONG);
                MethodHandle handleFn = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_value_get_string").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                int code = (int) handleFn.invokeExact(handle, outStr, outLen);
                if (code == 0) {
                    MemorySegment strPtr = outStr.get(ValueLayout.ADDRESS, 0);
                    long len = outLen.get(ValueLayout.JAVA_LONG, 0);
                    if (strPtr.equals(MemorySegment.NULL)) {
                        return Optional.empty();
                    }
                    byte[] bytes = strPtr.reinterpret(len).toArray(ValueLayout.JAVA_BYTE);
                    return Optional.of(new String(bytes, StandardCharsets.UTF_8));
                }
                return Optional.empty();
            } catch (Throwable t) {
                throw new RuntimeException("Failed to get string from value", t);
            }
        }

        public Value getField(String key) {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment cKey = arena.allocateFrom(key);
                MethodHandle handleFn = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_value_get_field").orElseThrow(),
                    FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
                );
                MemorySegment fieldPtr = (MemorySegment) handleFn.invokeExact(handle, cKey);
                if (fieldPtr.equals(MemorySegment.NULL)) {
                    return null;
                }
                return new Value(fieldPtr, false);
            } catch (Throwable t) {
                throw new RuntimeException("Failed to get field: " + key, t);
            }
        }

        @Override
        public void close() {
            if (owns && handle != null && !handle.equals(MemorySegment.NULL)) {
                try {
                    MethodHandle freeHandle = LINKER.downcallHandle(
                        LOOKUP.find("polyxml_value_free").orElseThrow(),
                        FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
                    );
                    freeHandle.invokeExact(handle);
                } catch (Throwable t) {
                    throw new RuntimeException("Failed to free native value", t);
                }
            }
        }
    }

    public static Value deserialize(byte[] xml, Schema schema) {
        Objects.requireNonNull(xml, "xml bytes cannot be null");
        Objects.requireNonNull(schema, "schema cannot be null");

        try (Arena arena = Arena.ofConfined()) {
            MemorySegment dataSeg = arena.allocate(xml.length);
            dataSeg.copyFrom(MemorySegment.ofArray(xml));

            MemorySegment outValue = arena.allocate(ValueLayout.ADDRESS);
            MethodHandle desHandle = LINKER.downcallHandle(
                LOOKUP.find("polyxml_deserialize").orElseThrow(),
                FunctionDescriptor.of(
                    ValueLayout.JAVA_INT,
                    ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG,
                    ValueLayout.ADDRESS,
                    ValueLayout.ADDRESS
                )
            );

            int code = (int) desHandle.invokeExact(dataSeg, (long) xml.length, schema.handle, outValue);
            if (code != 0) {
                throw new RuntimeException("PolyXML deserialization error (code: " + code + ")");
            }

            MemorySegment valPtr = outValue.get(ValueLayout.ADDRESS, 0);
            return new Value(valPtr, true);
        } catch (Throwable t) {
            if (t instanceof RuntimeException re) {
                throw re;
            }
            throw new RuntimeException("Failed to deserialize XML", t);
        }
    }

    public static Value deserialize(String xml, Schema schema) {
        Objects.requireNonNull(xml, "xml string cannot be null");
        return deserialize(xml.getBytes(StandardCharsets.UTF_8), schema);
    }

    public static byte[] serializeWithOptions(
        String rootName,
        Value value,
        Schema schema,
        int indent,
        Boolean enableNamespaces,
        Map<String, String> nsMap
    ) {
        Objects.requireNonNull(rootName, "rootName cannot be null");
        Objects.requireNonNull(value, "value cannot be null");
        Objects.requireNonNull(schema, "schema cannot be null");

        try (Arena arena = Arena.ofConfined()) {
            MemorySegment cRoot = arena.allocateFrom(rootName);
            MemorySegment outBytes = arena.allocate(ValueLayout.ADDRESS);
            MemorySegment outLen = arena.allocate(ValueLayout.JAVA_LONG);

            int enableNsInt = (enableNamespaces == null) ? -1 : (enableNamespaces ? 1 : 0);
            int nsCount = (nsMap != null) ? nsMap.size() : 0;

            MemorySegment prefixesPtr = MemorySegment.NULL;
            MemorySegment urisPtr = MemorySegment.NULL;

            if (nsCount > 0) {
                MemorySegment prefixesArr = arena.allocate(ValueLayout.ADDRESS, nsCount);
                MemorySegment urisArr = arena.allocate(ValueLayout.ADDRESS, nsCount);
                int idx = 0;
                for (Map.Entry<String, String> entry : nsMap.entrySet()) {
                    MemorySegment pSeg = arena.allocateFrom(entry.getKey() != null ? entry.getKey() : "");
                    MemorySegment uSeg = arena.allocateFrom(entry.getValue());
                    prefixesArr.setAtIndex(ValueLayout.ADDRESS, idx, pSeg);
                    urisArr.setAtIndex(ValueLayout.ADDRESS, idx, uSeg);
                    idx++;
                }
                prefixesPtr = prefixesArr;
                urisPtr = urisArr;
            }

            MethodHandle serHandle = LINKER.downcallHandle(
                LOOKUP.find("polyxml_serialize_with_options").orElseThrow(),
                FunctionDescriptor.of(
                    ValueLayout.JAVA_INT,
                    ValueLayout.ADDRESS,
                    ValueLayout.ADDRESS,
                    ValueLayout.ADDRESS,
                    ValueLayout.JAVA_INT,
                    ValueLayout.JAVA_INT,
                    ValueLayout.ADDRESS,
                    ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG,
                    ValueLayout.ADDRESS,
                    ValueLayout.ADDRESS
                )
            );

            int code = (int) serHandle.invokeExact(
                cRoot,
                value.handle,
                schema.handle,
                indent,
                enableNsInt,
                prefixesPtr,
                urisPtr,
                (long) nsCount,
                outBytes,
                outLen
            );

            if (code != 0) {
                throw new RuntimeException("PolyXML serialization error (code: " + code + ")");
            }

            MemorySegment bytesPtr = outBytes.get(ValueLayout.ADDRESS, 0);
            long len = outLen.get(ValueLayout.JAVA_LONG, 0);

            byte[] result = bytesPtr.reinterpret(len).toArray(ValueLayout.JAVA_BYTE);

            MethodHandle freeBytesHandle = LINKER.downcallHandle(
                LOOKUP.find("polyxml_bytes_free").orElseThrow(),
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.JAVA_LONG)
            );
            freeBytesHandle.invokeExact(bytesPtr, len);

            return result;
        } catch (Throwable t) {
            if (t instanceof RuntimeException re) {
                throw re;
            }
            throw new RuntimeException("Failed to serialize XML", t);
        }
    }

    public static byte[] serialize(String rootName, Value value, Schema schema, int indent) {
        return serializeWithOptions(rootName, value, schema, indent, null, null);
    }

    public static String serializeToString(String rootName, Value value, Schema schema, int indent) {
        return new String(serialize(rootName, value, schema, indent), StandardCharsets.UTF_8);
    }

    public static String version() {
        try {
            MethodHandle verHandle = LINKER.downcallHandle(
                LOOKUP.find("polyxml_version").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.ADDRESS)
            );
            MemorySegment strSeg = (MemorySegment) verHandle.invokeExact();
            return strSeg.getString(0);
        } catch (Throwable t) {
            return "0.9.0";
        }
    }

    private PolyXML() {}
}
