use std::error::Error;

use libuv_sys2::{self as ffi, uv_async_t};

use crate::Handle;

type Callback = Box<dyn FnMut() -> Result<(), Box<dyn Error>> + 'static>;

#[derive(Clone)]
pub struct AsyncHandle {
    handle: Handle<uv_async_t, Callback>,
}

unsafe impl Send for AsyncHandle {}
unsafe impl Sync for AsyncHandle {}

impl AsyncHandle {
    /// Registers a new callback on the Neovim event loop, returning an
    /// [`AsyncHandle`] which can be used to execute the callback from any
    /// thread. The callback will always be executed on the main thread.
    pub fn new<E, Cb>(mut callback: Cb) -> Result<Self, crate::Error>
    where
        E: Error + 'static,
        Cb: FnMut() -> Result<(), E> + 'static,
    {
        let mut handle = Handle::new(|uv_loop, handle| unsafe {
            ffi::uv_async_init(
                uv_loop,
                handle.as_mut_ptr(),
                Some(async_cb as _),
            )
        })?;

        let callback: Callback = Box::new(move || {
            // Type erase the callback by boxing its error.
            callback().map_err(|err| Box::new(err) as Box<dyn Error>)
        });

        unsafe { handle.set_data(callback) };

        Ok(Self { handle })
    }

    /// Wakes up the Neovim event loop and executes the callback associated to
    /// this handle. It is safe to call this function from any thread. The
    /// callback will be called on the main thread.
    ///
    /// NOTE: [libuv] will coalesce calls to [`AsyncHandle::send`], that is,
    /// not every call to it will yield an execution of the callback. For
    /// example: if [`AsyncHandle::send`] is called 5 times in a row before the
    /// callback is called, the callback will only be called once. If
    /// [`AsyncHandle::send`] is called again after the callback was called, it
    /// will be called again.
    ///
    /// [libuv]: https://libuv.org/
    pub fn send(&self) -> Result<(), crate::Error> {
        let retv =
            unsafe { ffi::uv_async_send(self.handle.as_ptr() as *mut _) };

        if retv < 0 {
            // TODO
            return Err(super::Error::CouldntTriggerAsyncHandle);
        }

        Ok(())
    }
}

extern "C" fn async_cb(ptr: *mut uv_async_t) {
    let handle: Handle<_, Callback> = unsafe { Handle::from_raw(ptr) };

    let callback = unsafe { handle.get_data() };

    if !callback.is_null() {
        let callback = unsafe { &mut *callback };

        if let Err(_err) = callback() {
            // TODO: what now?
        }
    }
}
