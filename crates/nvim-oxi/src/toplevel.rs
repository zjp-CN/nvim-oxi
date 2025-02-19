use luajit_bindings::{self as lua, ffi::*, macros::cstr};
use nvim_types::Function;

use crate::Result;

/// Same as [`print!`] but for the [`std::dbg!`] macro
///
/// # Examples
///
/// ```ignore
/// use nvim_oxi as nvim;
///
/// nvim::dbg!(Some("test"));
/// ```
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::print!("[{}:{}]", ::core::file!(), ::core::line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::print!("[{}:{}] {} = {:#?}",
                    ::core::file!(), ::core::line!(), ::core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

/// Binding to `vim.schedule`.
///
/// Schedules a callback to be invoked soon by the main event-loop. Useful to
/// avoid [`textlock`](https://neovim.io/doc/user/eval.html#textlock) or other
/// temporary restrictions.
pub fn schedule<F>(fun: F)
where
    F: FnOnce(()) -> Result<()> + 'static,
{
    // https://github.com/neovim/neovim/blob/master/src/nvim/lua/executor.c#L316
    //
    // Unfortunately the `nlua_schedule` C function is not exported, so we have
    // to call the Lua function instead.
    unsafe {
        lua::with_state(move |lstate| {
            // Put `vim.schedule` on the stack.
            lua_getglobal(lstate, cstr!("vim"));
            lua_getfield(lstate, -1, cstr!("schedule"));

            // Store the function in the registry and put a reference to it on
            // the stack.
            let fun = Function::from_fn_once(fun);
            lua_rawgeti(lstate, LUA_REGISTRYINDEX, fun.lua_ref());

            lua_call(lstate, 1, 0);

            // Pop `vim` off the stack and remove the function from the registry.
            lua_pop(lstate, 1);
            luaL_unref(lstate, LUA_REGISTRYINDEX, fun.lua_ref());
        })
    };
}
