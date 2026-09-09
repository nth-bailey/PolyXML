package io.polyxml;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.Objects;

/**
 * PolyXML: High-performance, polyglot XML data-binding engine for Java.
 * Powered by Java Foreign Function & Memory API (Project Panama).
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

        public SchemaBuilder addField(String name, String xmlName, FieldKind kind, ScalarType scalar) {
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment cName = arena.allocateFrom(name);
                MemorySegment cXmlName = arena.allocateFrom(xmlName);
                MethodHandle addHandle = LINKER.downcallHandle(
                    LOOKUP.find("polyxml_schema_builder_add_field").orElseThrow(),
                    FunctionDescriptor.ofVoid(
                        ValueLayout.ADDRESS,
                        ValueLayout.ADDRESS,
                        ValueLayout.ADDRESS,
                        ValueLayout.JAVA_INT,
                        ValueLayout.JAVA_INT
                    )
                );
                addHandle.invokeExact(handle, cName, cXmlName, kind.code, scalar.code);
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

    public static String version() {
        try {
            MethodHandle verHandle = LINKER.downcallHandle(
                LOOKUP.find("polyxml_version").orElseThrow(),
                FunctionDescriptor.of(ValueLayout.ADDRESS)
            );
            MemorySegment strSeg = (MemorySegment) verHandle.invokeExact();
            return strSeg.getString(0);
        } catch (Throwable t) {
            return "0.1.0";
        }
    }

    private PolyXML() {}
}
