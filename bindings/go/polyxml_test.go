package polyxml_test

import (
	"os"
	"strings"
	"testing"

	polyxml "github.com/polyxml/PolyXML/bindings/go"
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

func TestGoNamespaces(t *testing.T) {
	builder, err := polyxml.NewSchemaBuilder("Order")
	if err != nil {
		t.Fatalf("failed to create schema builder: %v", err)
	}

	builder.SetNamespace("https://example.com/orders")
	builder.AddField("id", "id", polyxml.FieldAttribute, polyxml.ScalarInt)
	builder.AddFieldWithNamespace("item", "item", polyxml.FieldElement, polyxml.ScalarString, "https://example.com/items")

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("failed to build schema: %v", err)
	}

	xml := []byte(`<ns0:Order xmlns:ns0="https://example.com/orders" xmlns:ns1="https://example.com/items" id="505"><ns1:item>GoGadget</ns1:item></ns0:Order>`)
	val, err := polyxml.Deserialize(xml, schema)
	if err != nil {
		t.Fatalf("failed to deserialize: %v", err)
	}

	idField := val.GetField("id")
	if idField == nil {
		t.Fatal("id field is nil")
	}
	id, err := idField.GetInt()
	if err != nil || id != 505 {
		t.Fatalf("expected id 505, got %d (err: %v)", id, err)
	}

	nsMap := map[string]string{
		"ord": "https://example.com/orders",
		"itm": "https://example.com/items",
	}

	enabled := true
	serialized, err := polyxml.SerializeWithOptions("Order", val, schema, 0, &enabled, nsMap)
	if err != nil {
		t.Fatalf("failed to serialize with options: %v", err)
	}

	s := string(serialized)
	if !strings.Contains(s, `xmlns:ord="https://example.com/orders"`) {
		t.Fatalf("missing xmlns:ord: %s", s)
	}
	if !strings.Contains(s, `<ord:Order`) {
		t.Fatalf("missing <ord:Order: %s", s)
	}
	if !strings.Contains(s, `<itm:item>GoGadget</itm:item>`) {
		t.Fatalf("missing <itm:item>: %s", s)
	}
}

func TestGoConformanceFixtures(t *testing.T) {
	data, err := os.ReadFile("testdata/atom_feed.xml")
	if err != nil {
		t.Fatalf("failed to read atom_feed.xml: %v", err)
	}

	builder, err := polyxml.NewSchemaBuilder("feed")
	if err != nil {
		t.Fatalf("failed to create schema builder: %v", err)
	}
	builder.SetNamespace("http://www.w3.org/2005/Atom")
	builder.AddField("title", "title", polyxml.FieldElement, polyxml.ScalarString)
	builder.AddField("id", "id", polyxml.FieldElement, polyxml.ScalarString)

	schema, err := builder.Build()
	if err != nil {
		t.Fatalf("failed to build schema: %v", err)
	}

	val, err := polyxml.Deserialize(data, schema)
	if err != nil {
		t.Fatalf("failed to deserialize atom feed: %v", err)
	}

	titleField := val.GetField("title")
	if titleField == nil {
		t.Fatal("title field is nil")
	}
	title, err := titleField.GetString()
	if err != nil || title != "PolyXML Engineering Updates" {
		t.Fatalf("expected title 'PolyXML Engineering Updates', got: %s", title)
	}

	nsMap := map[string]string{
		"": "http://www.w3.org/2005/Atom",
	}
	enabled := true
	serialized, err := polyxml.SerializeWithOptions("feed", val, schema, 2, &enabled, nsMap)
	if err != nil {
		t.Fatalf("failed to serialize atom feed: %v", err)
	}
	s := string(serialized)
	if !strings.Contains(s, `xmlns="http://www.w3.org/2005/Atom"`) {
		t.Fatalf("missing default xmlns in: %s", s)
	}
	if !strings.Contains(s, `<title>PolyXML Engineering Updates</title>`) {
		t.Fatalf("missing title in: %s", s)
	}
}
