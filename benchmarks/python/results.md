# Benchmark Results

| Workload | Engine | Type | Operation | Min Latency | Median Latency | Throughput | Peak RAM | Speedup |
| :--- | :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| Sensor Micro (~100B) | **PolyXML** | Typed Dataclass | Deserialize | 3.1 μs | 3.2 μs | 28.7 MB/s | 0.9 KiB | 13.9x |
| Sensor Micro (~100B) | **PolyXML (Pydantic)** | Typed Pydantic | Deserialize | 3.7 μs | 3.8 μs | 24.4 MB/s | 1.6 KiB | 11.8x |
| Sensor Micro (~100B) | **ElementTree** | Untyped DOM | Deserialize | 5.1 μs | 5.3 μs | 17.4 MB/s | 12.2 KiB | 8.4x |
| Sensor Micro (~100B) | **defusedxml** | Secure DOM | Deserialize | 8.8 μs | 9.0 μs | 10.2 MB/s | 20.9 KiB | 5.0x |
| Sensor Micro (~100B) | **lxml** | Untyped DOM | Deserialize | 3.2 μs | 3.3 μs | 27.6 MB/s | 0.1 KiB | 13.4x |
| Sensor Micro (~100B) | **lxml.objectify** | C Dynamic Object | Deserialize | 3.4 μs | 3.4 μs | 26.4 MB/s | 0.2 KiB | 12.8x |
| Sensor Micro (~100B) | **xmltodict** | Untyped Dict | Deserialize | 10.4 μs | 10.6 μs | 8.6 MB/s | 20.5 KiB | 4.2x |
| Sensor Micro (~100B) | **xsdata** | Typed Dataclass | Deserialize | 43.5 μs | 44.5 μs | 2.1 MB/s | 5.8 KiB | 1.0x |
| Sensor Micro (~100B) | **pydantic-xml** | Typed Pydantic | Deserialize | 16.5 μs | 17.1 μs | 5.4 MB/s | 4.0 KiB | 2.6x |
| Sensor Micro (~100B) | **declxml** | Declarative Dict | Deserialize | 10.3 μs | 10.9 μs | 8.7 MB/s | 12.0 KiB | 4.2x |
| Sensor Micro (~100B) | **untangle** | Dynamic Object | Deserialize | 20.0 μs | 20.7 μs | 4.5 MB/s | 14.6 KiB | 2.2x |
| Sensor Micro (~100B) | **PolyXML** | Typed Dataclass | Serialize | 1.7 μs | 1.8 μs | 56.8 MB/s | 0.4 KiB | 26.0x |
| Sensor Micro (~100B) | **PolyXML (Pydantic)** | Typed Pydantic | Serialize | 1.8 μs | 1.9 μs | 53.0 MB/s | 0.4 KiB | 24.3x |
| Sensor Micro (~100B) | **lxml.objectify** | C Dynamic Object | Serialize | 1.3 μs | 1.3 μs | 76.5 MB/s | 0.2 KiB | 35.0x |
| Sensor Micro (~100B) | **xmltodict** | Untyped Dict | Serialize | 14.9 μs | 15.4 μs | 6.5 MB/s | 3.3 KiB | 3.0x |
| Sensor Micro (~100B) | **xsdata** | Typed Dataclass | Serialize | 44.5 μs | 45.5 μs | 2.2 MB/s | 5.9 KiB | 1.0x |
| Sensor Micro (~100B) | **pydantic-xml** | Typed Pydantic | Serialize | 15.2 μs | 15.5 μs | 6.4 MB/s | 2.8 KiB | 2.9x |
| Sensor Micro (~100B) | **declxml** | Declarative Dict | Serialize | 24.2 μs | 24.8 μs | 4.0 MB/s | 6.4 KiB | 1.8x |
| Order Nested (10 items, 1014B) | **PolyXML** | Typed Dataclass | Deserialize | 34.7 μs | 35.6 μs | 27.9 MB/s | 4.1 KiB | 10.5x |
| Order Nested (10 items, 1014B) | **ElementTree** | Untyped DOM | Deserialize | 20.0 μs | 20.6 μs | 48.3 MB/s | 19.2 KiB | 18.2x |
| Order Nested (10 items, 1014B) | **defusedxml** | Secure DOM | Deserialize | 45.9 μs | 46.5 μs | 21.1 MB/s | 27.8 KiB | 7.9x |
| Order Nested (10 items, 1014B) | **lxml** | Untyped DOM | Deserialize | 15.9 μs | 16.4 μs | 60.9 MB/s | 0.1 KiB | 22.9x |
| Order Nested (10 items, 1014B) | **lxml.objectify** | C Dynamic Object | Deserialize | 16.1 μs | 16.3 μs | 60.1 MB/s | 0.2 KiB | 22.6x |
| Order Nested (10 items, 1014B) | **xmltodict** | Untyped Dict | Deserialize | 86.0 μs | 86.8 μs | 11.2 MB/s | 29.5 KiB | 4.2x |
| Order Nested (10 items, 1014B) | **xsdata** | Typed Dataclass | Deserialize | 364.2 μs | 381.8 μs | 2.7 MB/s | 15.6 KiB | 1.0x |
| Order Nested (10 items, 1014B) | **declxml** | Declarative Dict | Deserialize | 90.0 μs | 91.3 μs | 10.8 MB/s | 19.5 KiB | 4.0x |
| Order Nested (10 items, 1014B) | **untangle** | Dynamic Object | Deserialize | 77.6 μs | 78.7 μs | 12.5 MB/s | 32.2 KiB | 4.7x |
| Order Nested (10 items, 1014B) | **PolyXML** | Typed Dataclass | Serialize | 19.5 μs | 19.7 μs | 49.4 MB/s | 1.3 KiB | 22.4x |
| Order Nested (10 items, 1014B) | **lxml.objectify** | C Dynamic Object | Serialize | 6.5 μs | 6.5 μs | 148.0 MB/s | 1.1 KiB | 67.1x |
| Order Nested (10 items, 1014B) | **xmltodict** | Untyped Dict | Serialize | 135.9 μs | 138.0 μs | 7.1 MB/s | 9.9 KiB | 3.2x |
| Order Nested (10 items, 1014B) | **xsdata** | Typed Dataclass | Serialize | 437.5 μs | 452.9 μs | 2.2 MB/s | 12.6 KiB | 1.0x |
| Order Nested (10 items, 1014B) | **declxml** | Declarative Dict | Serialize | 167.7 μs | 171.6 μs | 5.8 MB/s | 21.3 KiB | 2.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML** | Typed Dataclass | Deserialize | 2.29 ms | 2.36 ms | 29.7 MB/s | 236.0 KiB | 9.9x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML (Pydantic)** | Typed Pydantic | Deserialize | 2.82 ms | 3.08 ms | 24.1 MB/s | 599.8 KiB | 8.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **ElementTree** | Untyped DOM | Deserialize | 1.05 ms | 1.09 ms | 64.8 MB/s | 743.0 KiB | 21.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **defusedxml** | Secure DOM | Deserialize | 2.63 ms | 2.72 ms | 25.8 MB/s | 751.3 KiB | 8.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml** | Untyped DOM | Deserialize | 738.9 μs | 766.5 μs | 91.8 MB/s | 0.1 KiB | 30.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml.objectify** | C Dynamic Object | Deserialize | 748.9 μs | 784.0 μs | 90.6 MB/s | 0.2 KiB | 30.2x |
| Catalog Batch (1,000 items, 69.5 KB) | **xmltodict** | Untyped Dict | Deserialize | 5.62 ms | 5.75 ms | 12.1 MB/s | 525.6 KiB | 4.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **xsdata** | Typed Dataclass | Deserialize | 22.61 ms | 23.21 ms | 3.0 MB/s | 469.4 KiB | 1.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **pydantic-xml** | Typed Pydantic | Deserialize | 11.36 ms | 11.67 ms | 6.0 MB/s | 1809.4 KiB | 2.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **declxml** | Declarative Dict | Deserialize | 4.87 ms | 5.15 ms | 13.9 MB/s | 848.6 KiB | 4.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **untangle** | Dynamic Object | Deserialize | 4.44 ms | 4.51 ms | 15.3 MB/s | 1488.9 KiB | 5.1x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML** | Typed Dataclass | Serialize | 1.16 ms | 1.19 ms | 58.3 MB/s | 69.9 KiB | 25.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml.objectify** | C Dynamic Object | Serialize | 372.5 μs | 376.6 μs | 180.9 MB/s | 69.6 KiB | 77.5x |
| Catalog Batch (1,000 items, 69.5 KB) | **xmltodict** | Untyped Dict | Serialize | 7.97 ms | 8.10 ms | 8.5 MB/s | 172.5 KiB | 3.6x |
| Catalog Batch (1,000 items, 69.5 KB) | **xsdata** | Typed Dataclass | Serialize | 28.87 ms | 29.31 ms | 2.3 MB/s | 171.3 KiB | 1.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **declxml** | Declarative Dict | Serialize | 9.66 ms | 9.83 ms | 7.0 MB/s | 694.5 KiB | 3.0x |
| Catalog Batch (10,000 items, 724.1 KB) | **PolyXML** | Typed Dataclass | Deserialize | 23.69 ms | 24.03 ms | 29.8 MB/s | 2072.6 KiB | 10.0x |
| Catalog Batch (10,000 items, 724.1 KB) | **ElementTree** | Untyped DOM | Deserialize | 13.34 ms | 13.59 ms | 53.0 MB/s | 7116.1 KiB | 17.8x |
| Catalog Batch (10,000 items, 724.1 KB) | **defusedxml** | Secure DOM | Deserialize | 29.19 ms | 29.45 ms | 24.2 MB/s | 7124.7 KiB | 8.1x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml** | Untyped DOM | Deserialize | 11.58 ms | 11.76 ms | 61.1 MB/s | 0.1 KiB | 20.5x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml.objectify** | C Dynamic Object | Deserialize | 11.60 ms | 11.82 ms | 61.0 MB/s | 0.2 KiB | 20.4x |
| Catalog Batch (10,000 items, 724.1 KB) | **xmltodict** | Untyped Dict | Deserialize | 56.77 ms | 57.53 ms | 12.5 MB/s | 4822.6 KiB | 4.2x |
| Catalog Batch (10,000 items, 724.1 KB) | **xsdata** | Typed Dataclass | Deserialize | 237.12 ms | 241.89 ms | 3.0 MB/s | 3254.9 KiB | 1.0x |
| Catalog Batch (10,000 items, 724.1 KB) | **PolyXML** | Typed Dataclass | Serialize | 12.09 ms | 12.21 ms | 58.1 MB/s | 719.5 KiB | 23.5x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml.objectify** | C Dynamic Object | Serialize | 4.05 ms | 4.09 ms | 173.5 MB/s | 724.1 KiB | 70.1x |
| Catalog Batch (10,000 items, 724.1 KB) | **xmltodict** | Untyped Dict | Serialize | 79.18 ms | 80.01 ms | 8.9 MB/s | 1645.3 KiB | 3.6x |
| Catalog Batch (10,000 items, 724.1 KB) | **xsdata** | Typed Dataclass | Serialize | 283.91 ms | 298.78 ms | 2.5 MB/s | 1634.2 KiB | 1.0x |
