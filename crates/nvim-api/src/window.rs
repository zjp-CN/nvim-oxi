use std::fmt;

use luajit_bindings::{self as lua, Poppable, Pushable};
use nvim_types::{
    self as nvim,
    Array,
    FromObject,
    FromObjectError,
    Function,
    Integer,
    Object,
    ToObject,
    WinHandle,
};
use serde::{Deserialize, Serialize};

use super::ffi::window::*;
use super::LUA_INTERNAL_CALL;
use super::{Buffer, TabPage};
use crate::Result;

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
/// A newtype struct wrapping a Neovim window. All the `nvim_win_*` functions
/// taking a window handle as their first argument are implemented as methods
/// on this object.
pub struct Window(pub(crate) WinHandle);

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Window").field(&self.0).finish()
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<H: Into<WinHandle>> From<H> for Window {
    fn from(handle: H) -> Self {
        Self(handle.into())
    }
}

impl From<Window> for Object {
    fn from(win: Window) -> Self {
        win.0.into()
    }
}

impl From<&Window> for Object {
    fn from(win: &Window) -> Self {
        win.0.into()
    }
}

impl FromObject for Window {
    fn from_obj(obj: Object) -> std::result::Result<Self, FromObjectError> {
        Ok(WinHandle::from_obj(obj)?.into())
    }
}

impl Poppable for Window {
    unsafe fn pop(
        lstate: *mut luajit_bindings::ffi::lua_State,
    ) -> std::result::Result<Self, lua::Error> {
        WinHandle::pop(lstate).map(Into::into)
    }
}

impl Window {
    /// Shorthand for [`api::get_current_win`](crate::api::get_current_win).
    #[inline(always)]
    pub fn current() -> Self {
        crate::get_current_win()
    }

    /// Binding to [`nvim_win_call`](https://neovim.io/doc/user/api.html#nvim_win_call()).
    ///
    /// Calls a function with this window as the temporary current window.
    pub fn call<R, F>(&self, fun: F) -> Result<R>
    where
        F: FnOnce(()) -> Result<R> + 'static,
        R: Pushable + FromObject,
    {
        let fun = Function::from_fn_once(fun);
        let mut err = nvim::Error::new();
        let obj = unsafe { nvim_win_call(self.0, fun.lua_ref(), &mut err) };

        err.into_err_or_flatten(move || {
            fun.remove_from_lua_registry();
            Ok(R::from_obj(obj)?)
        })
    }

    /// Binding to [`nvim_win_close`](https://neovim.io/doc/user/api.html#nvim_win_close()).
    ///
    /// Closes the window. Not allowed when
    /// [`textlock`](https://neovim.io/doc/user/eval.html#textlock) is active.
    pub fn close(self, force: bool) -> Result<()> {
        let mut err = nvim::Error::new();
        unsafe { nvim_win_close(self.0, force, &mut err) };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_del_var`](https://neovim.io/doc/user/api.html#nvim_win_del_var()).
    ///
    /// Removes a window-scoped (`w:`) variable.
    pub fn del_var(&mut self, name: &str) -> Result<()> {
        let mut err = nvim::Error::new();
        let name = nvim::String::from(name);
        unsafe { nvim_win_del_var(self.0, name.non_owning(), &mut err) };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_get_buf`](https://neovim.io/doc/user/api.html#nvim_win_get_buf()).
    ///
    /// Gets the current [`Buffer`] in the window.
    pub fn get_buf(&self) -> Result<Buffer> {
        let mut err = nvim::Error::new();
        let handle = unsafe { nvim_win_get_buf(self.0, &mut err) };
        err.into_err_or_else(|| handle.into())
    }

    /// Binding to [`nvim_win_get_cursor`](https://neovim.io/doc/user/api.html#nvim_win_get_cursor()).
    ///
    /// Gets the (1,0)-indexed cursor position in the window.
    pub fn get_cursor(&self) -> Result<(usize, usize)> {
        let mut err = nvim::Error::new();
        let arr = unsafe { nvim_win_get_cursor(self.0, &mut err) };
        err.into_err_or_flatten(|| {
            let mut iter = arr.into_iter();
            let line = usize::from_obj(iter.next().unwrap())?;
            let col = usize::from_obj(iter.next().unwrap())?;
            Ok((line, col))
        })
    }

    /// Binding to [`nvim_win_get_height`](https://neovim.io/doc/user/api.html#nvim_win_get_height()).
    ///
    /// Gets the window height as a count of rows.
    pub fn get_height(&self) -> Result<u32> {
        let mut err = nvim::Error::new();
        let height = unsafe { nvim_win_get_height(self.0, &mut err) };
        err.into_err_or_else(|| height.try_into().expect("always positive"))
    }

    /// Binding to [`nvim_win_get_number`](https://neovim.io/doc/user/api.html#nvim_win_get_number()).
    ///
    /// Gets the window number.
    pub fn get_number(&self) -> Result<u32> {
        let mut err = nvim::Error::new();
        let nr = unsafe { nvim_win_get_number(self.0, &mut err) };
        err.into_err_or_else(|| nr.try_into().expect("always positive"))
    }

    /// Binding to [`nvim_win_get_option`](https://neovim.io/doc/user/api.html#nvim_win_get_option()).
    ///
    /// Gets a window option value.
    pub fn get_option<Opt>(&self, name: &str) -> Result<Opt>
    where
        Opt: FromObject,
    {
        let mut err = nvim::Error::new();
        let name = nvim::String::from(name);
        let obj = unsafe {
            nvim_win_get_option(self.0, name.non_owning(), &mut err)
        };
        err.into_err_or_flatten(|| Ok(Opt::from_obj(obj)?))
    }

    /// Binding to [`nvim_win_get_position`](https://neovim.io/doc/user/api.html#nvim_win_get_position()).
    ///
    /// Gets the window position in display cells.
    pub fn get_position(&self) -> Result<(usize, usize)> {
        let mut err = nvim::Error::new();
        let arr = unsafe { nvim_win_get_position(self.0, &mut err) };
        err.into_err_or_flatten(|| {
            let mut iter = arr.into_iter();
            let line = usize::from_obj(iter.next().unwrap())?;
            let col = usize::from_obj(iter.next().unwrap())?;
            Ok((line, col))
        })
    }

    /// Binding to [`nvim_win_get_tabpage`](https://neovim.io/doc/user/api.html#nvim_win_get_tabpage()).
    ///
    /// Gets the window's `TabPage`.
    pub fn get_tabpage(&self) -> Result<TabPage> {
        let mut err = nvim::Error::new();
        let handle = unsafe { nvim_win_get_tabpage(self.0, &mut err) };
        err.into_err_or_else(|| handle.into())
    }

    /// Binding to [`nvim_win_get_var`](https://neovim.io/doc/user/api.html#nvim_win_get_var()).
    ///
    /// Gets a window-scoped (`w:`) variable.
    pub fn get_var<Var>(&self, name: &str) -> Result<Var>
    where
        Var: FromObject,
    {
        let mut err = nvim::Error::new();
        let name = nvim::String::from(name);
        let obj =
            unsafe { nvim_win_get_var(self.0, name.non_owning(), &mut err) };
        err.into_err_or_flatten(|| Ok(Var::from_obj(obj)?))
    }

    /// Binding to [`nvim_win_get_width`](https://neovim.io/doc/user/api.html#nvim_win_get_width()).
    ///
    /// Gets the window width as a number of columns.
    pub fn get_width(&self) -> Result<u32> {
        let mut err = nvim::Error::new();
        let width = unsafe { nvim_win_get_width(self.0, &mut err) };
        err.into_err_or_else(|| width.try_into().expect("always positive"))
    }

    /// Binding to [`nvim_win_hide`](https://neovim.io/doc/user/api.html#nvim_win_hide()).
    ///
    /// Closes the window and hides the buffer it contains.
    pub fn hide(self) -> Result<()> {
        let mut err = nvim::Error::new();
        unsafe { nvim_win_hide(self.0, &mut err) };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_is_valid`](https://neovim.io/doc/user/api.html#nvim_win_is_valid()).
    ///
    /// Checks if the window is valid.
    pub fn is_valid(&self) -> bool {
        unsafe { nvim_win_is_valid(self.0) }
    }

    /// Binding to [`nvim_win_set_buf`](https://neovim.io/doc/user/api.html#nvim_win_set_buf()).
    ///
    /// Sets `buffer` as the current buffer in the window.
    pub fn set_buf(&mut self, buffer: &Buffer) -> Result<()> {
        let mut err = nvim::Error::new();
        unsafe { nvim_win_set_buf(self.0, buffer.0, &mut err) };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_set_cursor`](https://neovim.io/doc/user/api.html#nvim_win_set_cursor()).
    ///
    /// Sets the (1,0)-indexed cursor in the window. This will scroll the
    /// window even if it's not the current one.
    pub fn set_cursor(&mut self, line: usize, col: usize) -> Result<()> {
        let mut err = nvim::Error::new();
        let pos = Array::from_iter([line as Integer, col as Integer]);
        unsafe { nvim_win_set_cursor(self.0, pos.non_owning(), &mut err) };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_set_height`](https://neovim.io/doc/user/api.html#nvim_win_set_height()).
    ///
    /// Sets the window height.
    pub fn set_height(&mut self, height: u32) -> Result<()> {
        let mut err = nvim::Error::new();
        unsafe { nvim_win_set_height(self.0, height.into(), &mut err) };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_set_option`](https://neovim.io/doc/user/api.html#nvim_win_set_option()).
    ///
    /// Sets a window option value. Passing `None` as value deletes the option
    /// (only works if there's a global fallback).
    pub fn set_option<Opt>(&mut self, name: &str, value: Opt) -> Result<()>
    where
        Opt: ToObject,
    {
        let mut err = nvim::Error::new();
        let name = nvim::String::from(name);
        unsafe {
            nvim_win_set_option(
                LUA_INTERNAL_CALL,
                self.0,
                name.non_owning(),
                value.to_obj()?.non_owning(),
                &mut err,
            )
        };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_set_var`](https://neovim.io/doc/user/api.html#nvim_win_set_var()).
    ///
    /// Sets a window-scoped (`w:`) variable.
    pub fn set_var<Var>(&mut self, name: &str, value: Var) -> Result<()>
    where
        Var: ToObject,
    {
        let mut err = nvim::Error::new();
        let name = nvim::String::from(name);
        unsafe {
            nvim_win_set_var(
                self.0,
                name.non_owning(),
                value.to_obj()?.non_owning(),
                &mut err,
            )
        };
        err.into_err_or_else(|| ())
    }

    /// Binding to [`nvim_win_set_width`](https://neovim.io/doc/user/api.html#nvim_win_set_width()).
    ///
    /// Sets the window width.
    pub fn set_width(&mut self, width: u32) -> Result<()> {
        let mut err = nvim::Error::new();
        unsafe { nvim_win_set_width(self.0, width.into(), &mut err) };
        err.into_err_or_else(|| ())
    }
}
