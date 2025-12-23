# GTS Macros

Procedural macros for GTS (Global Type System) schema generation from Rust structs.

## Overview

The `#[struct_to_gts_schema]` attribute macro serves **three purposes**:

1. **Compile-Time Validation** - Catches configuration errors before runtime
2. **Schema Generation** - Enables CLI-based JSON Schema file generation
3. **Runtime API** - Provides schema access, instance ID generation, and schema composition capabilities at runtime

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gts-macros = { path = "path/to/gts-rust/gts-macros" }
serde = { version = "1.0", features = ["derive"] }
```

## Quick Start

```rust
use gts_macros::struct_to_gts_schema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    schema_id = "gts.x.myapp.entities.user.v1~",
    description = "User entity with authentication information",
    properties = "id,email,name"
)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub internal_field: i32,  // Not included in schema
}

// Runtime usage:
fn example() {
    // Get the JSON Schema with $ref inheritance
    let schema_with_refs = User::GTS_JSON_SCHEMA_WITH_REFS;

    // Get the JSON Schema with inlined parent (currently identical to with_refs)
    let schema_inline = User::GTS_JSON_SCHEMA_INLINE;

    // Generate instance IDs (returns GtsInstanceId)
    let instance_id = User::make_gts_instance_id("123.v1");
    assert_eq!(instance_id.as_ref(), "gts.x.myapp.entities.user.v1~123.v1");
}
```

---

## Purpose 1: Compile-Time Validation

The macro validates your annotations at compile time, catching errors early.

### What Gets Validated

| Check | Description |
|-------|-------------|
| **Required parameters** | All of `dir_path`, `schema_id`, `description`, `properties` must be present |
| **Property existence** | Every property in the list must exist as a field in the struct |
| **Struct type** | Only structs with named fields are supported (no tuple structs) |

### Compile Error Examples

**Missing property:**
```rust
#[struct_to_gts_schema(
    dir_path = "schemas",
    schema_id = "gts.x.app.entities.user.v1~",
    description = "User",
    properties = "id,nonexistent"  // ❌ Error!
)]
pub struct User {
    pub id: String,
}
```
```
error: struct_to_gts_schema: Property 'nonexistent' not found in struct.
       Available fields: ["id"]
```

**Tuple struct:**
```rust
#[struct_to_gts_schema(/* ... */)]
pub struct Data(String);  // ❌ Tuple struct not supported
```
```
error: struct_to_gts_schema: Only structs with named fields are supported
```

---

## Purpose 2: Schema Generation

Generate JSON Schema files using the GTS CLI tool.

### Generate Schemas

```bash
# Using paths from macro (relative to source files)
gts generate-from-rust --source src/

# Override output directory
gts generate-from-rust --source src/ --output schemas/

# Exclude specific directories (can be used multiple times)
gts generate-from-rust --source . --exclude "tests/*" --exclude "examples/*"

# Using cargo
cargo run --bin gts -- generate-from-rust --source src/
```

### Excluding Files

The CLI provides multiple ways to exclude files from scanning:

**1. `--exclude` option** (supports glob patterns):
```bash
gts generate-from-rust --source . --exclude "tests/*" --exclude "benches/*"
```

**2. Auto-ignored directories**: The following directories are automatically skipped:
- `compile_fail/` - trybuild compile-fail tests

**3. `// gts:ignore` directive**: Add this comment at the top of any `.rs` file:
```rust
// gts:ignore
//! This file will be skipped by the CLI

use gts_macros::struct_to_gts_schema;
// ...
```

### What the CLI Does

1. Scans source files for `#[struct_to_gts_schema]` annotations
2. Extracts metadata (schema_id, description, properties)
3. Maps Rust types to JSON Schema types
4. Generates valid JSON Schema files at the specified `dir_path/<schema_id>.schema.json`

### Generated Schema Example

For the `User` struct above, generates `schemas/gts.x.myapp.entities.user.v1~.schema.json`:

```json
{
  "$id": "gts://gts.x.myapp.entities.user.v1~",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "User",
  "type": "object",
  "description": "User entity with authentication information",
  "properties": {
    "id": { "type": "string" },
    "email": { "type": "string" },
    "name": { "type": "string" }
  },
  "required": ["id", "email", "name"]
}
```

### Type Mapping

| Rust Type | JSON Schema Type | Format | Required |
|-----------|------------------|--------|----------|
| `String`, `&str` | `string` | - | Yes |
| `i8`-`i128`, `u8`-`u128` | `integer` | - | Yes |
| `f32`, `f64` | `number` | - | Yes |
| `bool` | `boolean` | - | Yes |
| `Vec<T>` | `array` | - | Yes |
| `Option<T>` | Same as `T` | - | **No** |
| `Uuid` | `string` | `uuid` | Yes |
| `DateTime`, `NaiveDateTime` | `string` | `date-time` | Yes |
| `NaiveDate` | `string` | `date` | Yes |
| `HashMap<K,V>`, `BTreeMap<K,V>` | `object` | - | Yes |

**Note**: `Option<T>` fields are not marked as `required` in the generated schema.

---

## Purpose 3: Runtime API

The macro generates associated constants, methods, and implements the `GtsSchema` trait for runtime use.

### `GTS_JSON_SCHEMA_WITH_REFS`

A compile-time constant containing the JSON Schema with `$id` set to `schema_id`. When inheritance is used (multiple segments in `schema_id`), this version uses `allOf` with `$ref` to reference the parent schema.

```rust
// Access the schema at runtime
let schema: &'static str = User::GTS_JSON_SCHEMA_WITH_REFS;

// Parse it if needed
let parsed: serde_json::Value = serde_json::from_str(schema).unwrap();
assert_eq!(parsed["$id"], "gts.x.myapp.entities.user.v1~");
```

### `GTS_JSON_SCHEMA_INLINE`

A compile-time constant containing the JSON Schema with the parent schema **inlined** (no `$ref`). Currently identical to `GTS_JSON_SCHEMA_WITH_REFS`, but will differ in future versions when true inlining is implemented.

```rust
// Access the inlined schema at runtime
let schema: &'static str = User::GTS_JSON_SCHEMA_INLINE;

// Parse it if needed
let parsed: serde_json::Value = serde_json::from_str(schema).unwrap();
assert_eq!(parsed["$id"], "gts.x.myapp.entities.user.v1~");
```

### `make_gts_instance_id(segment) -> GtsInstanceId`

Generate instance IDs by appending a segment to the schema ID. Returns a `gts::GtsInstanceId`
which can be used as a map key, compared, hashed, and serialized.

```rust
// Simple segment
let id = User::make_gts_instance_id("x.core.namespace.type.v1");
assert_eq!(id, "gts.x.myapp.entities.user.v1~x.core.namespace.type.v1");

// Multi-part segment
let id = User::make_gts_instance_id("x.bss.orders.commerce.v1");
assert_eq!(id, "gts.x.myapp.entities.user.v1~x.bss.orders.commerce.v1");

// Segment with wildcard
let id = User::make_gts_instance_id("a.b._.d.v1.0");
assert_eq!(id, "gts.x.myapp.entities.user.v1~a.b._.d.v1.0");

// Versioned segment
let id = User::make_gts_instance_id("vendor.pkg.namespace.instance.v2.1");
assert_eq!(id, "gts.x.myapp.entities.user.v1~vendor.pkg.namespace.instance.v2.1");

// Convert to String when needed
let id_string: String = id.into();

// Use as map key
use std::collections::HashMap;
let mut map: HashMap<gts::GtsInstanceId, String> = HashMap::new();
map.insert(User::make_gts_instance_id("key.v1"), "value".to_owned());
```

### Schema Composition & Inheritance (`GtsSchema` Trait)

The macro automatically implements the `GtsSchema` trait, enabling runtime schema composition for nested generic types. This allows you to compose schemas at runtime for complex type hierarchies like `BaseEventV1<AuditPayloadV1<PlaceOrderDataV1>>`.

```rust
use gts::GtsSchema;

// Get composed schema for nested type
let schema = BaseEventV1::<AuditPayloadV1<PlaceOrderDataV1>>::gts_schema_with_refs_allof();

// The schema will have proper nesting:
// - payload field contains AuditPayloadV1's schema
// - payload.data field contains PlaceOrderDataV1's schema
// - All with additionalProperties: false for type safety
```

**Generic Field Type Safety**: Generic fields (fields that accept nested types) automatically have `additionalProperties: false` set. This ensures:
- ✅ Only properly nested inherited structs can be used as values
- ✅ No arbitrary extra properties can be added to generic fields
- ✅ Type safety is enforced at the JSON Schema level

### Other Generated Constants

| Constant | Description |
|----------|-------------|
| `GTS_SCHEMA_ID` | The schema ID string |
| `GTS_SCHEMA_FILE_PATH` | The full file path for CLI generation (`{dir_path}/{schema_id}.schema.json`) |
| `GTS_SCHEMA_DESCRIPTION` | The description string |
| `GTS_SCHEMA_PROPERTIES` | Comma-separated property list |
| `GTS_JSON_SCHEMA_WITH_REFS` | JSON Schema with `allOf` + `$ref` for inheritance |
| `GTS_JSON_SCHEMA_INLINE` | JSON Schema with parent inlined (currently identical to WITH_REFS) |

---

## Macro Parameters

All parameters are **required** (4 total):

| Parameter | Description | Example |
|-----------|-------------|---------|
| `dir_path` | Output directory for generated schema | `"schemas"` |
| `schema_id` | GTS identifier | `"gts.x.app.entities.user.v1~"` |
| `description` | Human-readable description | `"User entity"` |
| `properties` | Comma-separated field list | `"id,email,name"` |

**Note**: The macro accepts exactly 4 parameters. Any other attributes will result in a compile error.

### GTS ID Format

```
gts.<vendor>.<package>.<namespace>.<type>.v<MAJOR>[.<MINOR>]~
```

Examples:
- `gts.x.core.iam.user.v1~` - IAM user schema
- `gts.x.commerce.orders.order.v1.0~` - Order schema with minor version

---

## Complete Example

### Define Structs

```rust
// src/models.rs
use gts_macros::struct_to_gts_schema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[struct_to_gts_schema(
    dir_path = "schemas",
    schema_id = "gts.x.shop.entities.product.v1~",
    description = "Product entity with pricing",
    properties = "id,name,price,in_stock"
)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub in_stock: bool,
    pub warehouse_id: String,  // Not in schema
}

#[derive(Debug, Serialize, Deserialize)]
#[struct_to_gts_schema(
    dir_path  = "schemas",
    schema_id = "gts.x.shop.entities.order.v1~",
    description = "Order entity",
    properties = "id,customer_id,total,status"
)]
pub struct Order {
    pub id: String,
    pub customer_id: String,
    pub total: f64,
    pub status: Option<String>,  // Optional field
}
```

### Generate Schemas

```bash
gts generate-from-rust --source src/
# Output:
#   Generated schema: gts.x.shop.entities.product.v1~ @ src/schemas/gts.x.shop.entities.product.v1~.schema.json
#   Generated schema: gts.x.shop.entities.order.v1~ @ src/schemas/gts.x.shop.entities.order.v1~.schema.json
```

### Use at Runtime

```rust
fn main() {
    // Access schema
    println!("Product schema: {}", Product::GTS_JSON_SCHEMA_WITH_REFS);

    // Generate instance IDs (returns GtsInstanceId)
    let product_id = Product::make_gts_instance_id("sku-12345.v1");
    let order_id = Order::make_gts_instance_id("ord-98765.v1");

    println!("Product ID: {}", product_id);
    // Output: gts.x.shop.entities.product.v1~sku-12345.v1

    println!("Order ID: {}", order_id);
    // Output: gts.x.shop.entities.order.v1~ord-98765.v1

    // Use as HashMap key
    use std::collections::HashMap;
    let mut inventory: HashMap<gts::GtsInstanceId, u32> = HashMap::new();
    inventory.insert(product_id, 100);
}
```

---

## Schema Inheritance & Compile-Time Guarantees

The macro supports **automatic inheritance detection** through chained GTS IDs and provides **compile-time validation** to prevent configuration errors.

### Inheritance Example

See `tests/inheritance_tests.rs` for a complete working example:

```rust
// Base event type (single segment - no inheritance)
#[struct_to_gts_schema(
    dir_path = "schemas",
    schema_id = "gts.x.core.events.type.v1~",
    description = "Base event type definition",
    properties = "event_type,id,tenant_id,sequence_id,payload"
)]
pub struct BaseEventV1<P> {
    #[serde(rename = "type")]
    pub event_type: String,
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub sequence_id: u64,
    pub payload: P,
}

// Extends BaseEventV1 via chained GTS ID
#[struct_to_gts_schema(
    dir_path = "schemas",
    schema_id = "gts.x.core.events.type.v1~x.core.audit.event.v1~",
    description = "Audit event with user context",
    properties = "user_agent,user_id,ip_address,data"
)]
pub struct AuditPayloadV1<D> {
    pub user_agent: String,
    pub user_id: Uuid,
    pub ip_address: String,
    pub data: D,
}

// Extends AuditPayloadV1 (3-level inheritance chain)
#[struct_to_gts_schema(
    dir_path = "schemas",
    schema_id = "gts.x.core.events.type.v1~x.core.audit.event.v1~x.marketplace.orders.purchase.v1~",
    description = "Order placement audit event",
    properties = "order_id,product_id"
)]
pub struct PlaceOrderDataV1 {
    pub order_id: Uuid,
    pub product_id: Uuid,
}
```

### Generated Schemas

**Single-segment schema** (no inheritance):
```json
{
  "$id": "gts://gts.x.core.events.type.v1~",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BaseEventV1",
  "type": "object",
  "description": "Base event type definition",
  "properties": { /* direct properties */ },
  "required": [ /* required fields */ ]
}
```

**Multi-segment schema** (with inheritance):
```json
{
  "$id": "gts://gts.x.core.events.type.v1~x.core.audit.event.v1~",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AuditPayloadV1",
  "type": "object",
  "description": "Audit event with user context",
  "allOf": [
    { "$ref": "gts://gts.x.core.events.type.v1~" },
    {
      "properties": {
        "payload": {
          "type": "object",
          "additionalProperties": false,
          "properties": { /* child-specific properties */ },
          "required": [ /* child-specific required fields */ ]
        }
      }
    }
  ]
}
```

**Important**: Generic fields (fields that accept nested types) automatically have `additionalProperties: false` set. This ensures that only properly nested inherited structs can be used, preventing arbitrary extra properties from being added to generic fields.

### Compile-Time Guarantees

The macro validates your configuration at compile time, preventing runtime errors:

| ✅ Guaranteed | ❌ Prevented |
|--------------|-------------|
| **All required attributes exist** | Missing `dir_path`, `schema_id`, `description`, or `properties` |
| **Properties exist in struct** | Referencing non-existent fields in `properties` list |
| **Valid struct types** | Tuple structs, unit structs, enums |
| **Single generic parameter** | Multiple type generics (prevents inheritance ambiguity) |
| **Valid GTS ID format** | Malformed schema identifiers |
| **Memory efficiency** | No unnecessary allocations in generated constants |
| **Strict generic field validation** | Generic fields have `additionalProperties: false` to ensure only nested inherited structs are allowed |

### Generic Fields and `additionalProperties`

When a struct has a generic type parameter (e.g., `BaseEventV1<P>` with field `payload: P`), the generated schema sets `additionalProperties: false` on that field's schema. This ensures:

- ✅ Only properly nested inherited structs can be used as values
- ✅ No arbitrary extra properties can be added to generic fields
- ✅ Type safety is enforced at the JSON Schema level

Example:
```json
{
  "properties": {
    "payload": {
      "type": "object",
      "additionalProperties": false,
      "properties": { /* nested schema */ }
    }
  }
}
```

### Schema Constants

The macro generates two schema variants with **zero runtime allocation**:

- **`GTS_JSON_SCHEMA_WITH_REFS`**: Uses `$ref` in `allOf` (most memory-efficient)
- **`GTS_JSON_SCHEMA_INLINE`**: Currently identical; true inlining requires runtime resolution

```rust
// Both are compile-time constants - no allocation at runtime!
let schema_with_refs = AuditPayloadV1::<()>::GTS_JSON_SCHEMA_WITH_REFS;
let schema_inline = AuditPayloadV1::<()>::GTS_JSON_SCHEMA_INLINE;

// Runtime schema resolution (when true inlining is needed)
use gts::GtsStore;
let store = GtsStore::new();
let inlined_schema = store.resolve_schema(&schema_with_refs)?;
```

---

## Security Features

The CLI includes security checks:

1. **Path traversal prevention** - Cannot write files outside the source repository
2. **File extension enforcement** - Both macro and CLI validate `.json` extension
3. **Canonicalization** - Resolves symbolic links to prevent escapes

---

## License

Apache-2.0

## See Also

- [GTS Specification](https://github.com/globaltypesystem/gts-spec)
- [GTS CLI Documentation](../gts-cli/README.md)
