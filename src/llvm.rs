extern crate inkwell;

/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type InterpreterFunc = unsafe extern "C" fn() -> i32;

