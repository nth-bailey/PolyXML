package polyxml_test

import (
	"strings"
	"testing"

	polyxml "github.com/nth-bailey/PolyXML/bindings/go"
)

func TestGoBindingsSignatures(t *testing.T) {
	_ = polyxml.FieldAttribute
	_ = polyxml.FieldElement
	_ = polyxml.FieldText

	_ = polyxml.ScalarString
	_ = polyxml.ScalarInt
	_ = polyxml.ScalarFloat
	_ = polyxml.ScalarBool
}

func TestGoDeserializationAndSerialization(t *testing.T) {
	builder, err := polyxml.NewSchemaBuilder("Sensor")
	if err != nil {
		t.Fatalf("failed to create schema builder: %v", err)
	}

	builder.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
	builder.AddField("name", "name", polyxml.FieldElement, polyxml.ScalarString)
	builder.AddField("reading", "reading", polyxml.FieldElement, polyxml.ScalarFloat)
	builder.AddField("calibrated", "calibrated", polyxml.FieldElement, polyxml.ScalarBool)

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("failed to build schema: %v", err)
	}

	xml := []byte(`<Sensor id="101"><name>Barometric Altimeter</name><reading>1013.25</reading><calibrated>true</calibrated></Sensor>`)

	val, err := polyxml.Deserialize(xml, schema)
	if err != nil {
		t.Fatalf("failed to deserialize XML: %v", err)
	}

	// Field assertions
	idField := val.GetField("id")
	if idField == nil {
		t.Fatal("id field is nil")
	}
	id, err := idField.GetInt()
	if err != nil || id != 101 {
		t.Fatalf("expected id 101, got %d (err: %v)", id, err)
	}

	nameField := val.GetField("name")
	name, err := nameField.GetString()
	if err != nil || name != "Barometric Altimeter" {
		t.Fatalf("expected name 'Barometric Altimeter', got %s", name)
	}

	readingField := val.GetField("reading")
	reading, err := readingField.GetFloat()
	if err != nil || reading != 1013.25 {
		t.Fatalf("expected reading 1013.25, got %f", reading)
	}

	calField := val.GetField("calibrated")
	calibrated, err := calField.GetBool()
	if err != nil || !calibrated {
		t.Fatalf("expected calibrated true, got %v", calibrated)
	}

	// Serialization round-trip
	serialized, err := polyxml.Serialize("Sensor", val, schema, 2)
	if err != nil {
		t.Fatalf("failed to serialize: %v", err)
	}

	serializedStr := string(serialized)
	if !strings.Contains(serializedStr, `id="101"`) {
		t.Fatalf("serialized output missing id attribute: %s", serializedStr)
	}
	if !strings.Contains(serializedStr, `<name>Barometric Altimeter</name>`) {
		t.Fatalf("serialized output missing name element: %s", serializedStr)
	}
}
