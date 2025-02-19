use derive_builder::Builder;
use nvim_types::{self as nvim, NonOwning, Object};

use crate::trait_utils::ToFunction;

/// Options passed to [`Buffer::set_keymap`](crate::Buffer::set_keymap)
/// and [`api::set_keymap`](crate::set_keymap).
#[derive(Clone, Debug, Default, PartialEq, Builder)]
#[builder(default, build_fn(private, name = "fallible_build"))]
pub struct SetKeymapOpts {
    #[builder(setter(custom))]
    callback: Object,

    #[builder(setter(custom))]
    desc: Object,

    /// Whether the keymap argument is an expression.
    #[builder(setter(strip_option))]
    expr: Option<bool>,

    /// Whether the right-hand side of the mapping shouldn't be remappable.
    #[builder(setter(strip_option))]
    noremap: Option<bool>,

    /// For buffer-local mappings, whether Neovim should wait for more
    /// characters to be typed if there's a global mapping that could also
    /// match. See `:h map-nowait` for more details.
    #[builder(setter(strip_option))]
    nowait: Option<bool>,

    /// When [`expr`](SetKeymapOptsBuilder::expr) is `true`, this option can be
    /// used to replace the keycodes in the resulting string (see
    /// [nvim_oxi::api::replace_termcodes](crate::replace_termcodes)).
    #[cfg(any(feature = "neovim-0-8", feature = "neovim-nightly"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(feature = "neovim-0-8", feature = "neovim-nightly")))
    )]
    #[builder(setter(strip_option))]
    replace_keycodes: Option<bool>,

    /// Whether to remap characters in the right-hand side by expanding the
    /// `<sid>` script tag.
    #[builder(setter(strip_option))]
    script: Option<bool>,

    /// Whether the keymap should be silent.
    #[builder(setter(strip_option))]
    silent: Option<bool>,

    /// If `true` setting the keymap fill fail if another keymap with the same
    /// left-hand side already exists.
    #[builder(setter(strip_option))]
    unique: Option<bool>,
}

impl SetKeymapOpts {
    #[inline(always)]
    /// Creates a new [`SetKeymapOptsBuilder`].
    pub fn builder() -> SetKeymapOptsBuilder {
        SetKeymapOptsBuilder::default()
    }
}

impl SetKeymapOptsBuilder {
    /// A function to call when the mapping is executed.
    pub fn callback<F>(&mut self, fun: F) -> &mut Self
    where
        F: ToFunction<(), ()>,
    {
        self.callback = Some(fun.to_obj());
        self
    }

    /// A description for the keymap.
    pub fn desc(&mut self, desc: &str) -> &mut Self {
        self.desc = Some(nvim::String::from(desc).into());
        self
    }

    pub fn build(&mut self) -> SetKeymapOpts {
        self.fallible_build().expect("never fails, all fields have defaults")
    }
}

#[derive(Default)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub(crate) struct KeyDict_keymap<'a> {
    desc: NonOwning<'a, Object>,
    expr: Object,
    script: Object,
    silent: Object,
    unique: Object,
    nowait: Object,
    noremap: Object,
    callback: NonOwning<'a, Object>,
    #[cfg(any(feature = "neovim-0-8", feature = "neovim-nightly"))]
    replace_keycodes: Object,
}

impl<'a> From<&'a SetKeymapOpts> for KeyDict_keymap<'a> {
    fn from(opts: &'a SetKeymapOpts) -> Self {
        Self {
            desc: opts.desc.non_owning(),
            expr: opts.expr.into(),
            script: opts.script.into(),
            silent: opts.silent.into(),
            unique: opts.unique.into(),
            nowait: opts.nowait.into(),
            noremap: opts.noremap.into(),
            callback: opts.callback.non_owning(),
            #[cfg(any(feature = "neovim-0-8", feature = "neovim-nightly"))]
            replace_keycodes: opts.replace_keycodes.into(),
        }
    }
}
