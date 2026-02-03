//! Benchmark for serialization and deserialization of Smithy shapes.
//!
//! Run with: cargo bench -p smithy4rs-json-codec --bench serde_benchmark

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use smithy4rs_core::{
    derive::SmithyShape,
    schema::prelude::{BOOLEAN, DOUBLE, FLOAT, INTEGER, LONG, STRING},
    serde::{ShapeBuilder, de::DeserializeWithSchema, serializers::SerializeWithSchema},
    smithy, IndexMap,
};
use smithy4rs_json_codec::{JsonDeserializer, JsonSerializer};

const SAMPLE_PAYLOAD: &[u8] = include_bytes!("sample_payload.json");

// ============================================================================
// Schema Definitions
// ============================================================================

smithy!("com.benchmark#TagList": {
    list TAG_LIST_SCHEMA {
        member: STRING
    }
});

smithy!("com.benchmark#IdList": {
    list ID_LIST_SCHEMA {
        member: STRING
    }
});

smithy!("com.benchmark#ScoreList": {
    list SCORE_LIST_SCHEMA {
        member: DOUBLE
    }
});

smithy!("com.benchmark#FlagList": {
    list FLAG_LIST_SCHEMA {
        member: BOOLEAN
    }
});

smithy!("com.benchmark#AttributeMap": {
    map ATTRIBUTE_MAP_SCHEMA {
        key: STRING
        value: STRING
    }
});

smithy!("com.benchmark#RecordMetadata": {
    structure RECORD_METADATA_SCHEMA {
        VERSION: STRING = "version"
        SOURCE: STRING = "source"
        CHECKSUM: STRING = "checksum"
        PRIORITY: INTEGER = "priority"
        WEIGHT: DOUBLE = "weight"
        FLAGS: FLAG_LIST_SCHEMA = "flags"
    }
});

smithy!("com.benchmark#BenchmarkRecord": {
    structure BENCHMARK_RECORD_SCHEMA {
        ID: STRING = "id"
        NAME: STRING = "name"
        DESCRIPTION: STRING = "description"
        CATEGORY: STRING = "category"
        PRICE: DOUBLE = "price"
        QUANTITY: INTEGER = "quantity"
        RATING: FLOAT = "rating"
        IS_AVAILABLE: BOOLEAN = "isAvailable"
        CREATED_AT: LONG = "createdAt"
        UPDATED_AT: LONG = "updatedAt"
        TAGS: TAG_LIST_SCHEMA = "tags"
        ATTRIBUTES: ATTRIBUTE_MAP_SCHEMA = "attributes"
        METADATA: RECORD_METADATA_SCHEMA = "metadata"
        RELATED_IDS: ID_LIST_SCHEMA = "relatedIds"
        SCORES: SCORE_LIST_SCHEMA = "scores"
    }
});

// ============================================================================
// Rust Structs
// ============================================================================

#[derive(SmithyShape, Clone, PartialEq)]
#[smithy_schema(RECORD_METADATA_SCHEMA)]
pub struct RecordMetadata {
    #[smithy_schema(VERSION)]
    pub version: String,
    #[smithy_schema(SOURCE)]
    pub source: String,
    #[smithy_schema(CHECKSUM)]
    pub checksum: Option<String>,
    #[smithy_schema(PRIORITY)]
    pub priority: Option<i32>,
    #[smithy_schema(WEIGHT)]
    pub weight: Option<f64>,
    #[smithy_schema(FLAGS)]
    pub flags: Option<Vec<bool>>,
}

#[derive(SmithyShape, Clone, PartialEq)]
#[smithy_schema(BENCHMARK_RECORD_SCHEMA)]
pub struct BenchmarkRecord {
    #[smithy_schema(ID)]
    pub id: String,
    #[smithy_schema(NAME)]
    pub name: String,
    #[smithy_schema(DESCRIPTION)]
    pub description: Option<String>,
    #[smithy_schema(CATEGORY)]
    pub category: String,
    #[smithy_schema(PRICE)]
    pub price: f64,
    #[smithy_schema(QUANTITY)]
    pub quantity: i32,
    #[smithy_schema(RATING)]
    pub rating: Option<f32>,
    #[smithy_schema(IS_AVAILABLE)]
    pub is_available: Option<bool>,
    #[smithy_schema(CREATED_AT)]
    pub created_at: Option<i64>,
    #[smithy_schema(UPDATED_AT)]
    pub updated_at: Option<i64>,
    #[smithy_schema(TAGS)]
    pub tags: Option<Vec<String>>,
    #[smithy_schema(ATTRIBUTES)]
    pub attributes: Option<IndexMap<String, String>>,
    #[smithy_schema(METADATA)]
    pub metadata: Option<RecordMetadata>,
    #[smithy_schema(RELATED_IDS)]
    pub related_ids: Option<Vec<String>>,
    #[smithy_schema(SCORES)]
    pub scores: Option<Vec<f64>>,
}

// ============================================================================
// Sample Data Loading
// ============================================================================

/// Loads a sample BenchmarkRecord from the shared sample_payload.json file.
fn load_sample_record() -> BenchmarkRecord {
    let mut deserializer = JsonDeserializer::new(SAMPLE_PAYLOAD);
    BenchmarkRecordBuilder::deserialize_with_schema(&BENCHMARK_RECORD_SCHEMA, &mut deserializer)
        .unwrap()
        .build()
        .unwrap()
}

// ============================================================================
// Benchmarks
// ============================================================================

fn benchmark_serialization(c: &mut Criterion) {
    let record = load_sample_record();

    // First, measure the payload size
    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    record
        .serialize_with_schema(&BENCHMARK_RECORD_SCHEMA, serializer)
        .unwrap();
    let payload_size = buf.len();
    println!(
        "Payload size: {} bytes ({:.2} KB)",
        payload_size,
        payload_size as f64 / 1024.0
    );

    let mut group = c.benchmark_group("serialization");
    group.throughput(Throughput::Bytes(payload_size as u64));

    group.bench_function("serialize_benchmark_record", |b| {
        b.iter(|| {
            let mut buf = Vec::with_capacity(payload_size);
            let serializer = JsonSerializer::new(&mut buf);
            record
                .serialize_with_schema(&BENCHMARK_RECORD_SCHEMA, serializer)
                .unwrap();
            black_box(buf)
        })
    });

    group.finish();
}

fn benchmark_deserialization(c: &mut Criterion) {
    let record = load_sample_record();

    // Serialize to get the JSON bytes
    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    record
        .serialize_with_schema(&BENCHMARK_RECORD_SCHEMA, serializer)
        .unwrap();
    let json_bytes = buf;
    let payload_size = json_bytes.len();

    let mut group = c.benchmark_group("deserialization");
    group.throughput(Throughput::Bytes(payload_size as u64));

    group.bench_function("deserialize_benchmark_record", |b| {
        b.iter(|| {
            let mut deserializer = JsonDeserializer::new(black_box(&json_bytes));
            let result = BenchmarkRecordBuilder::deserialize_with_schema(
                &BENCHMARK_RECORD_SCHEMA,
                &mut deserializer,
            )
            .unwrap()
            .build()
            .unwrap();
            black_box(result)
        })
    });

    group.finish();
}

fn benchmark_roundtrip(c: &mut Criterion) {
    let record = load_sample_record();

    // Get payload size
    let mut buf = Vec::new();
    let serializer = JsonSerializer::new(&mut buf);
    record
        .serialize_with_schema(&BENCHMARK_RECORD_SCHEMA, serializer)
        .unwrap();
    let payload_size = buf.len();

    let mut group = c.benchmark_group("roundtrip");
    group.throughput(Throughput::Bytes(payload_size as u64));

    group.bench_function("roundtrip_benchmark_record", |b| {
        b.iter(|| {
            // Serialize
            let mut buf = Vec::with_capacity(payload_size);
            let serializer = JsonSerializer::new(&mut buf);
            record
                .serialize_with_schema(&BENCHMARK_RECORD_SCHEMA, serializer)
                .unwrap();

            // Deserialize
            let mut deserializer = JsonDeserializer::new(&buf);
            let result = BenchmarkRecordBuilder::deserialize_with_schema(
                &BENCHMARK_RECORD_SCHEMA,
                &mut deserializer,
            )
            .unwrap()
            .build()
            .unwrap();
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_serialization,
    benchmark_deserialization,
    benchmark_roundtrip
);
criterion_main!(benches);
