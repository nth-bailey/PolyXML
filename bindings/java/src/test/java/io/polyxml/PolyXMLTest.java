package io.polyxml;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class PolyXMLTest {

    @Test
    public void testVersion() {
        String ver = PolyXML.version();
        assertNotNull(ver);
        assertFalse(ver.isEmpty());
    }

    @Test
    public void testEnumDefinitions() {
        assertEquals(0, PolyXML.FieldKind.ATTRIBUTE.ordinal());
        assertEquals(1, PolyXML.FieldKind.ELEMENT.ordinal());
        assertEquals(2, PolyXML.FieldKind.TEXT.ordinal());

        assertEquals(0, PolyXML.ScalarType.STRING.ordinal());
        assertEquals(1, PolyXML.ScalarType.INT.ordinal());
        assertEquals(2, PolyXML.ScalarType.FLOAT.ordinal());
        assertEquals(3, PolyXML.ScalarType.BOOL.ordinal());
    }
}
