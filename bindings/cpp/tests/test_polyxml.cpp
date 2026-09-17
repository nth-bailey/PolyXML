#include <cassert>
#include <iostream>
#include <string>

#include "polyxml.hpp"

void test_sensor_deserialization() {
    std::string xml = R"(<Sensor id="101"><name>Barometric Altimeter</name><reading>1013.25</reading><calibrated>true</calibrated></Sensor>)";

    auto schema = polyxml::SchemaBuilder("Sensor")
        .add_attribute("id", "id", POLYXML_SCALAR_INT)
        .add_element("name", "name", POLYXML_SCALAR_STRING)
        .add_element("reading", "reading", POLYXML_SCALAR_FLOAT)
        .add_element("calibrated", "calibrated", POLYXML_SCALAR_BOOL)
        .build();

    auto val = polyxml::deserialize(xml, schema);

    assert(val.get("id")->as_int().value() == 101);
    assert(val.get("name")->as_string().value() == "Barometric Altimeter");
    assert(val.get("reading")->as_float().value() == 1013.25);
    assert(val.get("calibrated")->as_bool().value() == true);

    std::cout << "PASS: test_sensor_deserialization\n";
}

void test_sensor_serialization_roundtrip() {
    std::string xml = R"(<Sensor id="42"><name>Gyro</name><reading>99.5</reading><calibrated>false</calibrated></Sensor>)";

    auto schema = polyxml::SchemaBuilder("Sensor")
        .add_attribute("id", "id", POLYXML_SCALAR_INT)
        .add_element("name", "name", POLYXML_SCALAR_STRING)
        .add_element("reading", "reading", POLYXML_SCALAR_FLOAT)
        .add_element("calibrated", "calibrated", POLYXML_SCALAR_BOOL)
        .build();

    auto val = polyxml::deserialize(xml, schema);
    std::string output = polyxml::serialize("Sensor", val, schema, 2);

    assert(output.find(R"(id="42")") != std::string::npos);
    assert(output.find("<name>Gyro</name>") != std::string::npos);
    assert(output.find("<calibrated>false</calibrated>") != std::string::npos);

    std::cout << "PASS: test_sensor_serialization_roundtrip\n";
}

void test_namespaced_serialization() {
    std::string xml = R"(<ns0:Order xmlns:ns0="https://example.com/orders" xmlns:ns1="https://example.com/items" id="999"><ns1:item>SuperGadget</ns1:item></ns0:Order>)";

    auto schema = polyxml::SchemaBuilder("Order")
        .set_namespace("https://example.com/orders")
        .add_attribute("id", "id", POLYXML_SCALAR_INT)
        .add_element("item", "item", POLYXML_SCALAR_STRING, "https://example.com/items")
        .build();

    auto val = polyxml::deserialize(xml, schema);
    assert(val.get("id")->as_int().value() == 999);
    assert(val.get("item")->as_string().value() == "SuperGadget");

    std::map<std::string, std::string> ns_map = {
        {"ord", "https://example.com/orders"},
        {"itm", "https://example.com/items"}
    };

    std::string output = polyxml::serialize_with_options("Order", val, schema, 0, true, ns_map);
    assert(output.find("xmlns:ord=\"https://example.com/orders\"") != std::string::npos);
    assert(output.find("xmlns:itm=\"https://example.com/items\"") != std::string::npos);
    assert(output.find("<ord:Order") != std::string::npos);
    assert(output.find("<itm:item>SuperGadget</itm:item>") != std::string::npos);

    std::cout << "PASS: test_namespaced_serialization\n";
}

int main() {
    std::cout << "Running PolyXML C++20 Test Suite...\n";
    test_sensor_deserialization();
    test_sensor_serialization_roundtrip();
    test_namespaced_serialization();
    std::cout << "All C++ tests passed successfully!\n";
    return 0;
}
