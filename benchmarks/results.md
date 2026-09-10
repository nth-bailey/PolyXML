# Benchmark Results

| Workload | Engine | Type | Operation | Min Latency | Median Latency | Throughput | Peak RAM | Speedup |
| :--- | :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| Sensor Micro (~100B) | **PolyXML** | Typed Dataclass | Deserialize | 2.8 μs | 3.0 μs | 31.8 MB/s | 0.6 KiB | 15.0x |
| Sensor Micro (~100B) | **PolyXML (Pydantic)** | Typed Pydantic | Deserialize | 3.9 μs | 4.1 μs | 23.1 MB/s | 1.6 KiB | 10.9x |
| Sensor Micro (~100B) | **ElementTree** | Untyped DOM | Deserialize | 4.9 μs | 5.1 μs | 18.3 MB/s | 11.9 KiB | 8.6x |
| Sensor Micro (~100B) | **defusedxml** | Secure DOM | Deserialize | 8.8 μs | 9.0 μs | 10.2 MB/s | 20.9 KiB | 4.8x |
| Sensor Micro (~100B) | **lxml** | Untyped DOM | Deserialize | 3.2 μs | 3.2 μs | 28.3 MB/s | 0.1 KiB | 13.4x |
| Sensor Micro (~100B) | **lxml.objectify** | C Dynamic Object | Deserialize | 3.4 μs | 3.4 μs | 26.6 MB/s | 0.2 KiB | 12.5x |
| Sensor Micro (~100B) | **xmltodict** | Untyped Dict | Deserialize | 10.3 μs | 10.5 μs | 8.7 MB/s | 20.5 KiB | 4.1x |
| Sensor Micro (~100B) | **xsdata** | Typed Dataclass | Deserialize | 42.3 μs | 43.7 μs | 2.1 MB/s | 5.8 KiB | 1.0x |
| Sensor Micro (~100B) | **pydantic-xml** | Typed Pydantic | Deserialize | 16.3 μs | 17.2 μs | 5.5 MB/s | 4.0 KiB | 2.6x |
| Sensor Micro (~100B) | **declxml** | Declarative Dict | Deserialize | 10.2 μs | 10.6 μs | 8.8 MB/s | 12.0 KiB | 4.1x |
| Sensor Micro (~100B) | **untangle** | Dynamic Object | Deserialize | 18.9 μs | 19.6 μs | 4.7 MB/s | 14.6 KiB | 2.2x |
| Sensor Micro (~100B) | **PolyXML** | Typed Dataclass | Serialize | 1.8 μs | 1.8 μs | 55.5 MB/s | 0.4 KiB | 36.3x |
| Sensor Micro (~100B) | **PolyXML (Pydantic)** | Typed Pydantic | Serialize | 1.9 μs | 1.9 μs | 52.2 MB/s | 0.4 KiB | 34.2x |
| Sensor Micro (~100B) | **lxml.objectify** | C Dynamic Object | Serialize | 1.3 μs | 1.3 μs | 75.8 MB/s | 0.2 KiB | 49.7x |
| Sensor Micro (~100B) | **xmltodict** | Untyped Dict | Serialize | 14.1 μs | 14.6 μs | 6.9 MB/s | 3.3 KiB | 4.5x |
| Sensor Micro (~100B) | **xsdata** | Typed Dataclass | Serialize | 63.7 μs | 71.2 μs | 1.5 MB/s | 5.9 KiB | 1.0x |
| Sensor Micro (~100B) | **pydantic-xml** | Typed Pydantic | Serialize | 15.5 μs | 21.0 μs | 6.3 MB/s | 2.8 KiB | 4.1x |
| Sensor Micro (~100B) | **declxml** | Declarative Dict | Serialize | 24.7 μs | 26.3 μs | 3.9 MB/s | 6.4 KiB | 2.6x |
| Order Nested (10 items, 1014B) | **PolyXML** | Typed Dataclass | Deserialize | 33.0 μs | 33.7 μs | 29.3 MB/s | 3.6 KiB | 10.9x |
| Order Nested (10 items, 1014B) | **ElementTree** | Untyped DOM | Deserialize | 20.1 μs | 20.7 μs | 48.0 MB/s | 19.5 KiB | 17.8x |
| Order Nested (10 items, 1014B) | **defusedxml** | Secure DOM | Deserialize | 45.1 μs | 45.6 μs | 21.5 MB/s | 27.8 KiB | 8.0x |
| Order Nested (10 items, 1014B) | **lxml** | Untyped DOM | Deserialize | 16.2 μs | 16.4 μs | 59.7 MB/s | 0.1 KiB | 22.2x |
| Order Nested (10 items, 1014B) | **lxml.objectify** | C Dynamic Object | Deserialize | 16.7 μs | 16.8 μs | 57.8 MB/s | 0.2 KiB | 21.5x |
| Order Nested (10 items, 1014B) | **xmltodict** | Untyped Dict | Deserialize | 85.5 μs | 87.9 μs | 11.3 MB/s | 29.5 KiB | 4.2x |
| Order Nested (10 items, 1014B) | **xsdata** | Typed Dataclass | Deserialize | 358.9 μs | 394.4 μs | 2.7 MB/s | 15.6 KiB | 1.0x |
| Order Nested (10 items, 1014B) | **declxml** | Declarative Dict | Deserialize | 90.9 μs | 96.3 μs | 10.6 MB/s | 19.2 KiB | 3.9x |
| Order Nested (10 items, 1014B) | **untangle** | Dynamic Object | Deserialize | 77.0 μs | 78.1 μs | 12.6 MB/s | 32.2 KiB | 4.7x |
| Order Nested (10 items, 1014B) | **PolyXML** | Typed Dataclass | Serialize | 23.0 μs | 23.3 μs | 42.0 MB/s | 1.2 KiB | 19.6x |
| Order Nested (10 items, 1014B) | **lxml.objectify** | C Dynamic Object | Serialize | 6.6 μs | 6.6 μs | 147.3 MB/s | 1.1 KiB | 68.6x |
| Order Nested (10 items, 1014B) | **xmltodict** | Untyped Dict | Serialize | 135.5 μs | 137.0 μs | 7.1 MB/s | 9.9 KiB | 3.3x |
| Order Nested (10 items, 1014B) | **xsdata** | Typed Dataclass | Serialize | 449.2 μs | 473.2 μs | 2.1 MB/s | 12.6 KiB | 1.0x |
| Order Nested (10 items, 1014B) | **declxml** | Declarative Dict | Serialize | 170.1 μs | 173.3 μs | 5.7 MB/s | 21.3 KiB | 2.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML** | Typed Dataclass | Deserialize | 2.28 ms | 4.05 ms | 29.8 MB/s | 235.7 KiB | 9.8x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML (Pydantic)** | Typed Pydantic | Deserialize | 2.91 ms | 3.34 ms | 23.4 MB/s | 596.2 KiB | 7.7x |
| Catalog Batch (1,000 items, 69.5 KB) | **ElementTree** | Untyped DOM | Deserialize | 1.05 ms | 1.14 ms | 64.6 MB/s | 743.0 KiB | 21.3x |
| Catalog Batch (1,000 items, 69.5 KB) | **defusedxml** | Secure DOM | Deserialize | 2.61 ms | 2.95 ms | 26.0 MB/s | 751.3 KiB | 8.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml** | Untyped DOM | Deserialize | 752.2 μs | 775.1 μs | 90.2 MB/s | 0.1 KiB | 29.7x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml.objectify** | C Dynamic Object | Deserialize | 767.7 μs | 807.8 μs | 88.4 MB/s | 0.2 KiB | 29.1x |
| Catalog Batch (1,000 items, 69.5 KB) | **xmltodict** | Untyped Dict | Deserialize | 5.57 ms | 5.70 ms | 12.2 MB/s | 525.6 KiB | 4.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **xsdata** | Typed Dataclass | Deserialize | 22.36 ms | 23.06 ms | 3.0 MB/s | 469.4 KiB | 1.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **pydantic-xml** | Typed Pydantic | Deserialize | 11.98 ms | 12.17 ms | 5.7 MB/s | 1809.4 KiB | 1.9x |
| Catalog Batch (1,000 items, 69.5 KB) | **declxml** | Declarative Dict | Deserialize | 4.89 ms | 5.14 ms | 13.9 MB/s | 848.4 KiB | 4.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **untangle** | Dynamic Object | Deserialize | 4.25 ms | 4.36 ms | 16.0 MB/s | 1488.9 KiB | 5.3x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML** | Typed Dataclass | Serialize | 1.39 ms | 1.41 ms | 48.6 MB/s | 70.9 KiB | 20.7x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml.objectify** | C Dynamic Object | Serialize | 373.1 μs | 375.5 μs | 180.7 MB/s | 69.6 KiB | 77.1x |
| Catalog Batch (1,000 items, 69.5 KB) | **xmltodict** | Untyped Dict | Serialize | 7.88 ms | 7.97 ms | 8.6 MB/s | 172.5 KiB | 3.7x |
| Catalog Batch (1,000 items, 69.5 KB) | **xsdata** | Typed Dataclass | Serialize | 28.78 ms | 29.11 ms | 2.3 MB/s | 171.3 KiB | 1.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **declxml** | Declarative Dict | Serialize | 9.77 ms | 9.92 ms | 6.9 MB/s | 694.5 KiB | 2.9x |
| Catalog Batch (10,000 items, 724.1 KB) | **PolyXML** | Typed Dataclass | Deserialize | 22.99 ms | 23.41 ms | 30.8 MB/s | 2079.0 KiB | 10.3x |
| Catalog Batch (10,000 items, 724.1 KB) | **ElementTree** | Untyped DOM | Deserialize | 13.59 ms | 13.95 ms | 52.0 MB/s | 7116.4 KiB | 17.4x |
| Catalog Batch (10,000 items, 724.1 KB) | **defusedxml** | Secure DOM | Deserialize | 28.56 ms | 29.67 ms | 24.8 MB/s | 7124.7 KiB | 8.3x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml** | Untyped DOM | Deserialize | 11.53 ms | 11.96 ms | 61.4 MB/s | 0.1 KiB | 20.6x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml.objectify** | C Dynamic Object | Deserialize | 11.42 ms | 11.97 ms | 61.9 MB/s | 0.2 KiB | 20.8x |
| Catalog Batch (10,000 items, 724.1 KB) | **xmltodict** | Untyped Dict | Deserialize | 56.11 ms | 57.01 ms | 12.6 MB/s | 4822.6 KiB | 4.2x |
| Catalog Batch (10,000 items, 724.1 KB) | **xsdata** | Typed Dataclass | Deserialize | 237.11 ms | 261.99 ms | 3.0 MB/s | 3254.9 KiB | 1.0x |
| Catalog Batch (10,000 items, 724.1 KB) | **PolyXML** | Typed Dataclass | Serialize | 14.87 ms | 15.32 ms | 47.2 MB/s | 719.3 KiB | 19.1x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml.objectify** | C Dynamic Object | Serialize | 4.14 ms | 4.21 ms | 169.8 MB/s | 724.1 KiB | 68.7x |
| Catalog Batch (10,000 items, 724.1 KB) | **xmltodict** | Untyped Dict | Serialize | 78.11 ms | 79.85 ms | 9.0 MB/s | 1645.3 KiB | 3.6x |
| Catalog Batch (10,000 items, 724.1 KB) | **xsdata** | Typed Dataclass | Serialize | 284.09 ms | 292.84 ms | 2.5 MB/s | 1634.2 KiB | 1.0x |
