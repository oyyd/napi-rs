use std::os::raw::{c_char, c_void};
use crate::{sys, Result, Value, Env, Function, ptr, check_status};

// TODO Should we pub these enums? or pub the sys mod directly?
pub use sys::napi_threadsafe_function_call_mode as napi_threadsafe_function_call_mode;
pub use sys::napi_threadsafe_function_release_mode as napi_threadsafe_function_release_mode;

pub trait ToJs: Copy + Clone {
  type Output;
  // TODO More than one argument
  type JsValue;

  fn resolve(&self, env: &mut Env, output: &mut Self::Output) -> Result<(u64, Value<Self::JsValue>)>;
}

#[derive(Debug, Clone, Copy)]
pub struct ThreadsafeFunction<T: ToJs> {
  raw_value: sys::napi_threadsafe_function,
  to_js: T,
}

unsafe impl<T: ToJs> Send for ThreadsafeFunction<T> {}

impl<T: ToJs> ThreadsafeFunction<T> {
  pub fn create(env: Env, func: Value<Function>, to_js: T) -> Result<Self> {
    let mut async_resource_name = ptr::null_mut();
    let s = "napi_rs_threadsafe_function";
    let status = unsafe {
      sys::napi_create_string_utf8(
        env.0,
        s.as_ptr() as *const c_char,
        s.len() as u64,
        &mut async_resource_name,
      )
    };
    check_status(status)?;

    let max_queue_size: u64 = 0;
    let initial_thread_count: u64 = 1;
    // let thread_finalize_data = ptr::null();
    // let napi_finalize = ptr::null();
    // let context = ptr::null();
    // let call_js_cb = ptr::null();
    let mut result = ptr::null_mut();
    let tsfn = ThreadsafeFunction {
      to_js,
      raw_value: result,
    };

    let status = unsafe {
      sys::napi_create_threadsafe_function(
        env.0,
        func.raw_value,
        ptr::null_mut(),
        async_resource_name,
        max_queue_size,
        initial_thread_count,
        ptr::null_mut(),
        None,
        Box::into_raw(Box::from(tsfn)) as *mut _ as *mut c_void,
        Some(call_js_cb::<T>),
        &mut result,
      )
    };
    check_status(status)?;

    // TODO
    Ok(ThreadsafeFunction {
      to_js,
      raw_value: result,
    })
  }

  pub fn call(&self, value: T::Output) -> Result<sys::napi_status> {
    Ok(unsafe {
      sys::napi_call_threadsafe_function(
        self.raw_value,
        Box::into_raw(Box::from(value)) as *mut _ as *mut c_void,
        // TODO
        napi_threadsafe_function_call_mode::napi_tsfn_blocking
      )
    })
  }

  pub fn acquire(&self) -> Result<sys::napi_status> {
    Ok(unsafe {
      sys::napi_acquire_threadsafe_function(
        self.raw_value
      )
    })
  }

  pub fn release(&self, mode: napi_threadsafe_function_release_mode) -> Result<sys::napi_status> {
    Ok(unsafe {
      sys::napi_release_threadsafe_function(
        self.raw_value,
        mode
      )
    })
  }
}

unsafe extern "C" fn call_js_cb<T: ToJs>(
  raw_env: sys::napi_env,
  js_callback: sys::napi_value,
  context: *mut c_void,
  data: *mut c_void
) {
  let mut env = Env::from_raw(raw_env);
  let mut recv = ptr::null_mut();
  sys::napi_get_undefined(raw_env, &mut recv);

  // TODO remove
  let mut test_value = ptr::null_mut();
  sys::napi_get_undefined(raw_env, &mut test_value);

  // TODO memory leak?
  let tsfn = Box::leak(Box::from_raw(context as *mut ThreadsafeFunction<T>));
  // TODO memory leak?
  let mut val = Box::from_raw(data as *mut T::Output);

  let ret = tsfn.to_js.resolve(&mut env, &mut val);
  let status;

  // Follow the convention of Node.js async callback.
  if ret.is_ok() {
    let (argv, js_value) = ret.unwrap();
    let js_null = env.get_null().unwrap();
    let values = [js_null.raw_value, js_value.raw_value];
    status = sys::napi_call_function(
      raw_env,
      recv,
      js_callback,
      argv + 1,
      values.as_ptr(),
      ptr::null_mut(),
    );
  } else {
    // TODO implement napi_create_error
    // let err = ret.err().unwrap();
    let mut err_obj = env.create_object().unwrap();
    status = sys::napi_call_function(
      raw_env,
      recv,
      js_callback,
      1,
      &mut err_obj.raw_value,
      ptr::null_mut(),
    );
  }

  debug_assert!(
    status == sys::napi_status::napi_ok,
    "CallJsCB failed"
  );
}
