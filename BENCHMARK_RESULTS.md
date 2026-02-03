# Benchmark Comparison: smithy4rs vs smithy-rs

## Results

| Metric | smithy4rs | smithy-rs | Difference |
|--------|-----------|-----------|------------|
| **Serialization** | | | |
| Throughput | 1.68 GiB/s | 1.62 GiB/s | smithy4rs +4% |
| Latency | 2.74 µs | 2.84 µs | |
| **Deserialization** | | | |
| Throughput | 826 MiB/s | 506 MiB/s | smithy4rs +63% |
| Latency | 5.69 µs | 9.29 µs | |
| **Roundtrip** | | | |
| Throughput | 542 MiB/s | 379 MiB/s | smithy4rs +43% |
| Latency | 8.67 µs | 12.40 µs | |

## Architecture

| Aspect | smithy4rs | smithy-rs |
|--------|-----------|-----------|
| Serialization | Schema-driven (`JsonSerializer`) | Code-generated `serde::Serialize` |
| JSON Writer | Custom (`jiter` based) | `serde_json` |
| Deserialization | Schema-driven (`JsonDeserializer`) | Token-based parser (`aws-smithy-json`) |
| JSON Parser | `jiter` | Custom token iterator |
| Payload Size | 4,931 bytes | 4,927 bytes |

## Implementation Details

### smithy4rs

Uses the same schema-driven approach for both serialization and deserialization:

```rust
// Serialization
let serializer = JsonSerializer::new(&mut buf);
record.serialize_with_schema(&BENCHMARK_RECORD_SCHEMA, serializer)

// Deserialization
let mut deserializer = JsonDeserializer::new(&json_bytes);
BenchmarkRecordBuilder::deserialize_with_schema(&BENCHMARK_RECORD_SCHEMA, &mut deserializer)
```

### smithy-rs

Uses **two completely different approaches** for serialization vs deserialization:

| Operation | Implementation | Library |
|-----------|----------------|---------|
| Serialization | `codegen-serde` generates `serde::Serialize` | `serde_json` |
| Deserialization | `aws-smithy-json` token-based parser | Custom |

**Serialization** uses a wrapper pattern with generated `serde::Serialize` impl:

```rust
// Benchmark code
serde_json::to_vec(&record.serialize_ref(&settings))

// Generated code (src/serde/shape_benchmark_record.rs)
impl serde::Serialize for ConfigurableSerdeRef<'_, BenchmarkRecord> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("BenchmarkRecord", 15)?;
        s.serialize_field("id", &inner.id.serialize_ref(self.settings))?;
        s.serialize_field("name", &inner.name.serialize_ref(self.settings))?;
        // ... etc
        s.end()
    }
}
```

**Deserialization** uses `aws-smithy-json` token iterator (NOT serde):

```rust
// Benchmark code
protocol_serde::shape_benchmark_record::de_benchmark_record_payload(&json_bytes)

// Generated code (src/protocol_serde/shape_benchmark_record.rs)
pub fn de_benchmark_record_payload(value: &[u8]) -> Result<BenchmarkRecord, DeserializeError> {
    let mut tokens = aws_smithy_json::deserialize::json_token_iter(value).peekable();
    // Manual token-by-token parsing...
}
```

**Note:** The `codegen-serde` module only generates `serde::Serialize` (not `Deserialize`). This means the serialization and deserialization benchmarks use fundamentally different code paths in smithy-rs.

## Running the Benchmarks

### smithy4rs

```bash
cd /Volumes/workplace/new_framework_dev/smithy4rs
cargo bench -p smithy4rs-json-codec --bench serde_benchmark
```

### smithy-rs

```bash
# Generate code (if needed)
cd /Volumes/workplace/new_framework_dev/smithy-rs
./gradlew :codegen-client-test:smithyBuild

# Run benchmark
cd codegen-client-test/build/smithyprojections/codegen-client-test/benchmark_serde/rust-client-codegen
cargo bench --features serde --bench serde_benchmark
```

## Test Data

Both benchmarks use identical `sample_payload.json` (~5KB) stored locally in each project:
- **smithy4rs**: `json-codec/benches/sample_payload.json`
- **smithy-rs**: `codegen-client-test/build/.../benchmark_serde/rust-client-codegen/benches/sample_payload.json`
