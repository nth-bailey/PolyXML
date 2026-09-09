# Benchmark Results

| Workload | Engine | Type | Operation | Min Latency | Median Latency | Throughput | Peak RAM | Speedup |
| :--- | :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| Sensor Micro (~100B) | **PolyXML** | Typed Dataclass | Deserialize | 2.8 μs | 2.9 μs | 32.6 MB/s | 0.5 KiB | 15.9x |
| Sensor Micro (~100B) | **PolyXML (Pydantic)** | Typed Pydantic | Deserialize | 3.0 μs | 3.1 μs | 30.1 MB/s | 1.4 KiB | 14.7x |
| Sensor Micro (~100B) | **ElementTree** | Untyped DOM | Deserialize | 5.3 μs | 5.8 μs | 16.8 MB/s | 12.2 KiB | 8.2x |
| Sensor Micro (~100B) | **defusedxml** | Secure DOM | Deserialize | 8.7 μs | 9.2 μs | 10.3 MB/s | 20.9 KiB | 5.0x |
| Sensor Micro (~100B) | **lxml** | Untyped DOM | Deserialize | 3.2 μs | 3.2 μs | 28.1 MB/s | 0.1 KiB | 13.7x |
| Sensor Micro (~100B) | **lxml.objectify** | C Dynamic Object | Deserialize | 3.4 μs | 3.5 μs | 26.1 MB/s | 0.2 KiB | 12.7x |
| Sensor Micro (~100B) | **xmltodict** | Untyped Dict | Deserialize | 10.5 μs | 10.9 μs | 8.5 MB/s | 20.5 KiB | 4.2x |
| Sensor Micro (~100B) | **xsdata** | Typed Dataclass | Deserialize | 43.8 μs | 45.4 μs | 2.0 MB/s | 5.8 KiB | 1.0x |
| Sensor Micro (~100B) | **pydantic-xml** | Typed Pydantic | Deserialize | 16.8 μs | 17.4 μs | 5.3 MB/s | 4.0 KiB | 2.6x |
| Sensor Micro (~100B) | **declxml** | Declarative Dict | Deserialize | 10.4 μs | 10.9 μs | 8.6 MB/s | 12.1 KiB | 4.2x |
| Sensor Micro (~100B) | **untangle** | Dynamic Object | Deserialize | 20.0 μs | 20.6 μs | 4.5 MB/s | 14.6 KiB | 2.2x |
| Sensor Micro (~100B) | **PolyXML** | Typed Dataclass | Serialize | 1.1 μs | 1.1 μs | 91.8 MB/s | 0.4 KiB | 42.0x |
| Sensor Micro (~100B) | **PolyXML (Pydantic)** | Typed Pydantic | Serialize | 1.2 μs | 1.3 μs | 82.4 MB/s | 0.4 KiB | 37.7x |
| Sensor Micro (~100B) | **lxml.objectify** | C Dynamic Object | Serialize | 1.2 μs | 1.3 μs | 78.4 MB/s | 0.2 KiB | 35.9x |
| Sensor Micro (~100B) | **xmltodict** | Untyped Dict | Serialize | 15.0 μs | 15.3 μs | 6.5 MB/s | 3.3 KiB | 3.0x |
| Sensor Micro (~100B) | **xsdata** | Typed Dataclass | Serialize | 44.5 μs | 45.8 μs | 2.2 MB/s | 5.9 KiB | 1.0x |
| Sensor Micro (~100B) | **pydantic-xml** | Typed Pydantic | Serialize | 15.6 μs | 15.9 μs | 6.2 MB/s | 2.8 KiB | 2.8x |
| Sensor Micro (~100B) | **declxml** | Declarative Dict | Serialize | 24.8 μs | 25.5 μs | 3.9 MB/s | 6.3 KiB | 1.8x |
| Order Nested (10 items, 1014B) | **PolyXML** | Typed Dataclass | Deserialize | 20.9 μs | 21.2 μs | 46.3 MB/s | 2.4 KiB | 17.0x |
| Order Nested (10 items, 1014B) | **ElementTree** | Untyped DOM | Deserialize | 20.1 μs | 20.2 μs | 48.1 MB/s | 19.3 KiB | 17.6x |
| Order Nested (10 items, 1014B) | **defusedxml** | Secure DOM | Deserialize | 45.3 μs | 45.8 μs | 21.3 MB/s | 27.8 KiB | 7.8x |
| Order Nested (10 items, 1014B) | **lxml** | Untyped DOM | Deserialize | 16.5 μs | 16.8 μs | 58.7 MB/s | 0.1 KiB | 21.5x |
| Order Nested (10 items, 1014B) | **lxml.objectify** | C Dynamic Object | Deserialize | 16.6 μs | 19.4 μs | 58.4 MB/s | 0.2 KiB | 21.4x |
| Order Nested (10 items, 1014B) | **xmltodict** | Untyped Dict | Deserialize | 85.5 μs | 89.1 μs | 11.3 MB/s | 29.5 KiB | 4.1x |
| Order Nested (10 items, 1014B) | **xsdata** | Typed Dataclass | Deserialize | 354.5 μs | 410.6 μs | 2.7 MB/s | 15.6 KiB | 1.0x |
| Order Nested (10 items, 1014B) | **declxml** | Declarative Dict | Deserialize | 90.3 μs | 91.6 μs | 10.7 MB/s | 19.5 KiB | 3.9x |
| Order Nested (10 items, 1014B) | **untangle** | Dynamic Object | Deserialize | 78.0 μs | 84.5 μs | 12.4 MB/s | 32.2 KiB | 4.5x |
| Order Nested (10 items, 1014B) | **PolyXML** | Typed Dataclass | Serialize | 11.7 μs | 12.3 μs | 82.3 MB/s | 1.2 KiB | 37.4x |
| Order Nested (10 items, 1014B) | **lxml.objectify** | C Dynamic Object | Serialize | 6.6 μs | 6.7 μs | 145.6 MB/s | 1.1 KiB | 66.2x |
| Order Nested (10 items, 1014B) | **xmltodict** | Untyped Dict | Serialize | 136.4 μs | 143.6 μs | 7.1 MB/s | 9.9 KiB | 3.2x |
| Order Nested (10 items, 1014B) | **xsdata** | Typed Dataclass | Serialize | 438.9 μs | 441.3 μs | 2.2 MB/s | 12.6 KiB | 1.0x |
| Order Nested (10 items, 1014B) | **declxml** | Declarative Dict | Serialize | 172.2 μs | 215.7 μs | 5.6 MB/s | 21.3 KiB | 2.5x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML** | Typed Dataclass | Deserialize | 1.36 ms | 1.37 ms | 50.0 MB/s | 195.5 KiB | 16.1x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML (Pydantic)** | Typed Pydantic | Deserialize | 1.98 ms | 2.01 ms | 34.2 MB/s | 579.3 KiB | 11.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **ElementTree** | Untyped DOM | Deserialize | 1.07 ms | 1.09 ms | 63.7 MB/s | 742.9 KiB | 20.5x |
| Catalog Batch (1,000 items, 69.5 KB) | **defusedxml** | Secure DOM | Deserialize | 2.57 ms | 2.59 ms | 26.4 MB/s | 751.3 KiB | 8.5x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml** | Untyped DOM | Deserialize | 776.2 μs | 901.3 μs | 87.4 MB/s | 0.1 KiB | 28.1x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml.objectify** | C Dynamic Object | Deserialize | 785.0 μs | 820.8 μs | 86.5 MB/s | 0.2 KiB | 27.8x |
| Catalog Batch (1,000 items, 69.5 KB) | **xmltodict** | Untyped Dict | Deserialize | 5.50 ms | 5.61 ms | 12.3 MB/s | 525.6 KiB | 4.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **xsdata** | Typed Dataclass | Deserialize | 21.83 ms | 22.03 ms | 3.1 MB/s | 469.4 KiB | 1.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **pydantic-xml** | Typed Pydantic | Deserialize | 10.91 ms | 11.06 ms | 6.2 MB/s | 1809.4 KiB | 2.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **declxml** | Declarative Dict | Deserialize | 4.86 ms | 5.00 ms | 14.0 MB/s | 848.4 KiB | 4.5x |
| Catalog Batch (1,000 items, 69.5 KB) | **untangle** | Dynamic Object | Deserialize | 4.18 ms | 4.25 ms | 16.2 MB/s | 1488.9 KiB | 5.2x |
| Catalog Batch (1,000 items, 69.5 KB) | **PolyXML** | Typed Dataclass | Serialize | 727.7 μs | 755.6 μs | 92.6 MB/s | 69.3 KiB | 39.5x |
| Catalog Batch (1,000 items, 69.5 KB) | **lxml.objectify** | C Dynamic Object | Serialize | 383.8 μs | 388.7 μs | 175.6 MB/s | 69.6 KiB | 74.9x |
| Catalog Batch (1,000 items, 69.5 KB) | **xmltodict** | Untyped Dict | Serialize | 7.84 ms | 8.02 ms | 8.6 MB/s | 172.5 KiB | 3.7x |
| Catalog Batch (1,000 items, 69.5 KB) | **xsdata** | Typed Dataclass | Serialize | 28.75 ms | 29.25 ms | 2.3 MB/s | 171.3 KiB | 1.0x |
| Catalog Batch (1,000 items, 69.5 KB) | **declxml** | Declarative Dict | Serialize | 9.44 ms | 9.53 ms | 7.1 MB/s | 694.5 KiB | 3.0x |
| Catalog Batch (10,000 items, 724.1 KB) | **PolyXML** | Typed Dataclass | Deserialize | 13.48 ms | 13.60 ms | 52.5 MB/s | 2032.3 KiB | 16.8x |
| Catalog Batch (10,000 items, 724.1 KB) | **ElementTree** | Untyped DOM | Deserialize | 12.22 ms | 12.31 ms | 57.9 MB/s | 7116.4 KiB | 18.6x |
| Catalog Batch (10,000 items, 724.1 KB) | **defusedxml** | Secure DOM | Deserialize | 27.42 ms | 27.61 ms | 25.8 MB/s | 7124.7 KiB | 8.3x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml** | Untyped DOM | Deserialize | 10.07 ms | 10.14 ms | 70.2 MB/s | 0.1 KiB | 22.5x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml.objectify** | C Dynamic Object | Deserialize | 10.08 ms | 10.44 ms | 70.2 MB/s | 0.2 KiB | 22.5x |
| Catalog Batch (10,000 items, 724.1 KB) | **xmltodict** | Untyped Dict | Deserialize | 55.80 ms | 55.98 ms | 12.7 MB/s | 4822.6 KiB | 4.1x |
| Catalog Batch (10,000 items, 724.1 KB) | **xsdata** | Typed Dataclass | Deserialize | 226.94 ms | 227.77 ms | 3.1 MB/s | 3254.9 KiB | 1.0x |
| Catalog Batch (10,000 items, 724.1 KB) | **PolyXML** | Typed Dataclass | Serialize | 7.30 ms | 7.55 ms | 96.2 MB/s | 719.5 KiB | 38.5x |
| Catalog Batch (10,000 items, 724.1 KB) | **lxml.objectify** | C Dynamic Object | Serialize | 3.89 ms | 3.94 ms | 180.7 MB/s | 724.1 KiB | 72.3x |
| Catalog Batch (10,000 items, 724.1 KB) | **xmltodict** | Untyped Dict | Serialize | 78.50 ms | 81.54 ms | 8.9 MB/s | 1645.3 KiB | 3.6x |
| Catalog Batch (10,000 items, 724.1 KB) | **xsdata** | Typed Dataclass | Serialize | 280.80 ms | 281.51 ms | 2.5 MB/s | 1634.2 KiB | 1.0x |
