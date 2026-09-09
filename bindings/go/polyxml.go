package polyxml

/*
#cgo CFLAGS: -I../../crates/polyxml-c/include
#include "polyxml.h"
#include <stdlib.h>
*/
import "C"
import (
	"errors"
	"runtime"
	"unsafe"
)

type FieldKind int

const (
	FieldAttribute FieldKind = C.POLYXML_FIELD_ATTRIBUTE
	FieldElement   FieldKind = C.POLYXML_FIELD_ELEMENT
	FieldText      FieldKind = C.POLYXML_FIELD_TEXT
)

type ScalarType int

const (
	ScalarString   ScalarType = C.POLYXML_SCALAR_STRING
	ScalarInt      ScalarType = C.POLYXML_SCALAR_INT
	ScalarFloat    ScalarType = C.POLYXML_SCALAR_FLOAT
	ScalarBool     ScalarType = C.POLYXML_SCALAR_BOOL
	ScalarDecimal  ScalarType = C.POLYXML_SCALAR_DECIMAL
	ScalarDate     ScalarType = C.POLYXML_SCALAR_XML_DATE
	ScalarDateTime ScalarType = C.POLYXML_SCALAR_XML_DATETIME
	ScalarAny      ScalarType = C.POLYXML_SCALAR_ANY
)

type Schema struct {
	ptr *C.polyxml_schema_t
}

type SchemaBuilder struct {
	ptr *C.polyxml_schema_builder_t
}

func NewSchemaBuilder(name string) (*SchemaBuilder, error) {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	ptr := C.polyxml_schema_builder_create(cName)
	if ptr == nil {
		return nil, errors.New("failed to create schema builder")
	}
	return &SchemaBuilder{ptr: ptr}, nil
}

func (b *SchemaBuilder) AddField(name, xmlName string, kind FieldKind, scalar ScalarType) {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	cXmlName := C.CString(xmlName)
	defer C.free(unsafe.Pointer(cXmlName))

	C.polyxml_schema_builder_add_field(
		b.ptr,
		cName,
		cXmlName,
		C.polyxml_field_kind_t(kind),
		C.polyxml_scalar_type_t(scalar),
	)
}

func (b *SchemaBuilder) Build() (*Schema, error) {
	ptr := C.polyxml_schema_builder_build(b.ptr)
	b.ptr = nil
	if ptr == nil {
		return nil, errors.New("failed to build schema")
	}
	schema := &Schema{ptr: ptr}
	runtime.SetFinalizer(schema, func(s *Schema) {
		if s.ptr != nil {
			C.polyxml_schema_free(s.ptr)
			s.ptr = nil
		}
	})
	return schema, nil
}

type Value struct {
	ptr  *C.polyxml_value_t
	owns bool
}

func Deserialize(xml []byte, schema *Schema) (*Value, error) {
	if len(xml) == 0 {
		return nil, errors.New("empty XML data")
	}

	var outVal *C.polyxml_value_t
	code := C.polyxml_deserialize(
		(*C.uint8_t)(unsafe.Pointer(&xml[0])),
		C.size_t(len(xml)),
		schema.ptr,
		&outVal,
	)

	if code != C.POLYXML_OK || outVal == nil {
		return nil, errors.New("failed to deserialize XML")
	}

	val := &Value{ptr: outVal, owns: true}
	runtime.SetFinalizer(val, func(v *Value) {
		if v.owns && v.ptr != nil {
			C.polyxml_value_free(v.ptr)
			v.ptr = nil
		}
	})
	return val, nil
}

func Serialize(rootName string, val *Value, schema *Schema, indent int) ([]byte, error) {
	cRoot := C.CString(rootName)
	defer C.free(unsafe.Pointer(cRoot))

	var outBytes *C.uint8_t
	var outLen C.size_t

	code := C.polyxml_serialize(
		cRoot,
		val.ptr,
		schema.ptr,
		C.int(indent),
		&outBytes,
		&outLen,
	)

	if code != C.POLYXML_OK || outBytes == nil {
		return nil, errors.New("failed to serialize XML")
	}

	res := C.GoBytes(unsafe.Pointer(outBytes), C.int(outLen))
	C.polyxml_bytes_free(outBytes, outLen)
	return res, nil
}

func (v *Value) GetField(key string) *Value {
	cKey := C.CString(key)
	defer C.free(unsafe.Pointer(cKey))

	sub := C.polyxml_value_get_field(v.ptr, cKey)
	if sub == nil {
		return nil
	}
	return &Value{ptr: (*C.polyxml_value_t)(unsafe.Pointer(sub)), owns: false}
}

func (v *Value) GetInt() (int64, error) {
	var out C.int64_t
	code := C.polyxml_value_get_int(v.ptr, &out)
	if code != C.POLYXML_OK {
		return 0, errors.New("value is not an int")
	}
	return int64(out), nil
}

func (v *Value) GetFloat() (float64, error) {
	var out C.double
	code := C.polyxml_value_get_float(v.ptr, &out)
	if code != C.POLYXML_OK {
		return 0, errors.New("value is not a float")
	}
	return float64(out), nil
}

func (v *Value) GetBool() (bool, error) {
	var out C.bool
	code := C.polyxml_value_get_bool(v.ptr, &out)
	if code != C.POLYXML_OK {
		return false, errors.New("value is not a bool")
	}
	return bool(out), nil
}

func (v *Value) GetString() (string, error) {
	var outStr *C.char
	var outLen C.size_t
	code := C.polyxml_value_get_string(v.ptr, &outStr, &outLen)
	if code != C.POLYXML_OK || outStr == nil {
		return "", errors.New("value is not a string")
	}
	return C.GoStringN(outStr, C.int(outLen)), nil
}

func (v *Value) IsNull() bool {
	return bool(C.polyxml_value_is_null(v.ptr))
}

func Version() string {
	return C.GoString(C.polyxml_version())
}
