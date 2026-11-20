//! `serialize-toml-c` — native TOML serializer backend for `lib-serialize`.
//!
//! `lib-serialize` is the `.hnl` contract: `serialize.serialize(v, fmt)` /
//! `serialize.deserialize(s, fmt)` dispatch on the format name or on the
//! `serialize`/`deserialize` arrow fields of a serializer record produced
//! by `serialize.toml()`. This crate holds the native impls (extracted
//! from `plugin-lib-stdlib`'s `ops::toml`), registered as
//! `toml_c.serialize` / `toml_c.deserialize` ops via an [`OpRegistry`].

use std::sync::Arc;

use handl_lib_serialize::{Decoder, Encoder, SerializeError, Serializer};
use handl_ops::{Op, OpContext, OpError, OpRegistry};
use serde_json::Value;
use toml::Value as TomlValue;

/// The handl source declaring the `toml_c` `op` interface — the single
/// source of truth for the toml_c API surface.
pub const TOML_C_HNL: &str = include_str!("index.hnl");

/// The op names declared in `index.hnl` and bound by
/// [`toml_c_op_registry`].
pub const TOML_C_OPS: &[&str] = &["toml_c.serialize", "toml_c.deserialize"];

// ---------------------------------------------------------------------------
// Toml codec (backward compat for the lib-serialize SerializerRegistry)
// ---------------------------------------------------------------------------

/// Combined TOML codec.
#[derive(Debug, Clone, Copy, Default)]
pub struct Toml;

/// TOML encoder.
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlEncoder;

/// TOML decoder.
#[derive(Debug, Clone, Copy, Default)]
pub struct TomlDecoder;

impl Encoder<TomlValue> for Toml {
    type Output = String;

    fn encode(&self, value: &TomlValue) -> Result<Self::Output, SerializeError> {
        toml::to_string(value).map_err(|e| SerializeError::new(e.to_string()))
    }
}

impl Decoder<str, TomlValue> for Toml {
    fn decode(&self, input: &str) -> Result<TomlValue, SerializeError> {
        toml::from_str(input).map_err(|e| SerializeError::new(e.to_string()))
    }
}

impl Serializer for Toml {
    fn serialize(&self, value: &Value) -> Result<String, SerializeError> {
        toml::to_string(value).map_err(|e| SerializeError::new(e.to_string()))
    }

    fn deserialize(&self, input: &str) -> Result<Value, SerializeError> {
        toml::from_str(input).map_err(|e| SerializeError::new(e.to_string()))
    }
}

impl Encoder<TomlValue> for TomlEncoder {
    type Output = String;

    fn encode(&self, value: &TomlValue) -> Result<Self::Output, SerializeError> {
        Toml.encode(value)
    }
}

impl Decoder<str, TomlValue> for TomlDecoder {
    fn decode(&self, input: &str) -> Result<TomlValue, SerializeError> {
        Toml.decode(input)
    }
}

// ---------------------------------------------------------------------------
// Function impls (the handls — Rust bodies for the `op` declarations)
// ---------------------------------------------------------------------------

/// The Rust handl for `op "toml_c.serialize" : Value -> String`.
///
/// Serializes a value to a TOML string. This is the eval-time `Function`
/// impl registered under the op name `"toml_c.serialize"` in the
/// [`OpRegistry`].
#[derive(Debug, Clone, Copy)]
pub struct TomlCSerialize;

impl Op for TomlCSerialize {
    fn call(&self, args: &[Value], _ctx: &mut dyn OpContext) -> Result<Value, OpError> {
        if args.len() != 1 {
            return Err(OpError::TypeMismatch(format!(
                "expected {} args, got {}",
                1,
                args.len()
            )));
        }
        let s = toml::to_string(&args[0])
            .map_err(|e| OpError::Other(format!("toml_c.serialize error: {e}")))?;
        Ok(Value::String(s))
    }
}

/// The Rust handl for `op "toml_c.deserialize" : String -> Value`.
///
/// Parses a TOML string into a value. This is the eval-time `Function`
/// impl registered under the op name `"toml_c.deserialize"` in the
/// [`OpRegistry`].
#[derive(Debug, Clone, Copy)]
pub struct TomlCDeserialize;

impl Op for TomlCDeserialize {
    fn call(&self, args: &[Value], _ctx: &mut dyn OpContext) -> Result<Value, OpError> {
        if args.len() != 1 {
            return Err(OpError::TypeMismatch(format!(
                "expected {} args, got {}",
                1,
                args.len()
            )));
        }
        let s = args[0]
            .as_str()
            .ok_or_else(|| OpError::Other("toml_c.deserialize expects a string".to_string()))?;
        let value: Value = toml::from_str(s)
            .map_err(|e| OpError::Other(format!("toml_c.deserialize error: {e}")))?;
        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// Op Registry
// ---------------------------------------------------------------------------

/// Build the [`OpRegistry`] for `toml_c`, registering the Rust handls under
/// their op names.
///
/// This registry maps:
/// - `"toml_c.serialize"` → [`TomlCSerialize`]
/// - `"toml_c.deserialize"` → [`TomlCDeserialize`]
pub fn toml_c_op_registry() -> OpRegistry {
    let mut registry = OpRegistry::new();
    registry.register("toml_c.serialize", Arc::new(TomlCSerialize));
    registry.register("toml_c.deserialize", Arc::new(TomlCDeserialize));
    registry
}
