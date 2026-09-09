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

int main() {
    std::cout << "Running PolyXML C++20 Test Suite...\n";
    test_sensor_deserialization();
    test_sensor_serialization_roundtrip();
    std::cout << "All C++ tests passed successfully!\n";
    return 0;
}
