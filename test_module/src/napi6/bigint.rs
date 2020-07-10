use std::convert::TryFrom;
use napi::{CallContext, JsBigint, Result, JsNumber, JsObject};

#[js_function(0)]
pub fn test_create_bigint_from_i64(ctx: CallContext) -> Result<JsBigint> {
  ctx.env.create_bigint_from_i64(i64::max_value())
}

#[js_function(0)]
pub fn test_create_bigint_from_u64(ctx: CallContext) -> Result<JsBigint> {
  ctx.env.create_bigint_from_u64(u64::max_value())
}

#[js_function(0)]
pub fn test_create_bigint_from_words(ctx: CallContext) -> Result<JsBigint> {
  ctx.env.create_bigint_from_words(true, vec![u64::max_value(), u64::max_value()])
}

#[js_function(1)]
pub fn test_get_bigint_i64(ctx: CallContext) -> Result<JsNumber> {
  let js_bigint = ctx.get::<JsBigint>(0)?;
  let val = i64::try_from(js_bigint)?;
  ctx.env.create_int32(val as i32)
}

#[js_function(1)]
pub fn test_get_bigint_u64(ctx: CallContext) -> Result<JsNumber> {
  let js_bigint = ctx.get::<JsBigint>(0)?;
  let val = u64::try_from(js_bigint)?;
  ctx.env.create_int32(val as i32)
}

#[js_function(0)]
pub fn test_get_bigint_words(ctx: CallContext) -> Result<JsObject> {
  let js_bigint = ctx.env.create_bigint_from_words(true, vec![i64::max_value() as u64, i64::max_value() as u64])?;
  let mut js_arr = ctx.env.create_array_with_length(2)?;
  let words = js_bigint.get_words(true)?;
  js_arr.set_number_indexed_property(
    ctx.env.create_int64(0)?,
    ctx.env.create_bigint_from_u64(words[0])?
  )?;
  js_arr.set_number_indexed_property(
    ctx.env.create_int64(1)?,
    ctx.env.create_bigint_from_u64(words[1])?
  )?;
  Ok(js_arr)
}
