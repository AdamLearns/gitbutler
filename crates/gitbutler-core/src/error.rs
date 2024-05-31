//! ## How-To
//!
//! This is a primer on how to use the types provided here.
//!
//! Generally, if you do not care about attaching an error code for error-classification or
//! and/or messages that may show up in the user interface, read no further. Just use `anyhow`
//! or `thiserror` like before.
//!
//! ### Adding Context
//!
//! The [`Context`] type is the richest context we may attach to either `anyhow` errors or `thiserror`,
//! albeit using a different mechanism. This context is maintained as the error propagates and the
//! context higest up the error chain, the one most recently added, can be used by higher layers
//! of the GitButler application. Currently, a [`Context::message`] is shown in the user-interface,
//! whereas [`Context::code`] can be provided to help the user interface to make decisions, as it uses
//! the code for classifying errors.
//!
//! #### With `anyhow`
//!
//! The basis is an error without context, just by using `anyhow` in any way.
//!
//!```rust
//!# use anyhow::bail;
//! fn f() -> anyhow::Result<()> {
//!    bail!("internal information")
//! }
//!```
//!
//! Adding context is as easy as using the `context()` method on any `Result` or [`anyhow::Error`].
//! This can be a [`Code`], which automatically uses the message provided previously in the
//! frontend (note, though, that this is an implementation detail, which may change).
//! It serves as marker to make these messages show up, even if the code is [`Code::Unknown`].
//!
//!```rust
//!# use anyhow::{anyhow};
//!# use gitbutler_core::error::Code;
//! fn f() -> anyhow::Result<()> {
//!    return Err(anyhow!("user information").context(Code::Unknown))
//! }
//!```
//!
//! Finally, it's also possible to specify the user-message by using a [`Context`].
//!
//!```rust
//!# use anyhow::{anyhow};
//!# use gitbutler_core::error::{Code, Context};
//! fn f() -> anyhow::Result<()> {
//!    return Err(anyhow!("internal information").context(Context::new_static(Code::Unknown, "user information")))
//! }
//!```
//!
//! #### Backtraces and `anyhow`
//!
//! Backtraces are automatically collected when `anyhow` errors are instantiated, as long as the
//! `RUST_BACKTRACE` variable is set.
//!
//! #### With `thiserror`
//!
//! `thiserror` doesn't have a mechanism for generic context, and if it's needed the error must be converted to `anyhow::Error`.
//!
//! By default, `thiserror` instances have no context.
//!
//! ### Assuring Context
//!
//! Currently, the consumers of errors with context are quite primitive and thus rely on `anyhow`
//! to collect and find context hidden in the error chain.
//! To make that work, it's important that `thiserror` based errors never silently convert into
//! `anyhow::Error`, as the context-consumers wouldn't find the context anymore.
//!
//! To prevent issues around this, make sure that relevant methods use the [`Error`] type provided
//! here. It is made to only automatically convert from types that have context information.
//! Those who have not will need to be converted by hand using [`Error::from_err()`].
use std::borrow::Cow;
use std::fmt::Debug;

/// A unique code that consumers of the API may rely on to identify errors.
///
/// ### Important
///
/// **Only add variants if a consumer, like the *frontend*, is actually using them**.
/// Remove variants when no longer in use.
///
/// In practice, it should match its [frontend counterpart](https://github.com/gitbutlerapp/gitbutler/blob/fa973fd8f1ae8807621f47601803d98b8a9cf348/app/src/lib/backend/ipc.ts#L5).
#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub enum Code {
    /// Much like a catch-all error code. It shouldn't be attached explicitly unless
    /// a message is provided as well as part of a [`Context`].
    #[default]
    Unknown,
    Validation,
    ProjectGitAuth,
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Code::Unknown => "errors.unknown",
            Code::Validation => "errors.validation",
            Code::ProjectGitAuth => "errors.projects.git.auth",
        };
        f.write_str(code)
    }
}

/// A context for classifying errors.
///
/// It provides a [`Code`], which may be [unknown](Code::Unknown), and a `message` which explains
/// more about the problem at hand.
#[derive(Default, Debug, Clone)]
pub struct Context {
    /// The classification of the error.
    pub code: Code,
    /// A description of what went wrong, if available.
    pub message: Option<Cow<'static, str>>,
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_deref().unwrap_or("Something went wrong"))
    }
}

impl From<Code> for Context {
    fn from(code: Code) -> Self {
        Context {
            code,
            message: None,
        }
    }
}

impl Context {
    /// Create a new instance with `code` and an owned `message`.
    pub fn new(message: impl Into<String>) -> Self {
        Context {
            code: Code::Unknown,
            message: Some(Cow::Owned(message.into())),
        }
    }

    /// Create a new instance with `code` and a statically known `message`.
    pub const fn new_static(code: Code, message: &'static str) -> Self {
        Context {
            code,
            message: Some(Cow::Borrowed(message)),
        }
    }

    /// Adjust the `code` of this instance to the given one.
    pub fn with_code(mut self, code: Code) -> Self {
        self.code = code;
        self
    }
}

mod private {
    pub trait Sealed {}
}

/// A way to obtain attached Code or context information from `anyhow` contexts, so that
/// the more complete information is preferred.
pub trait AnyhowContextExt: private::Sealed {
    /// Return our custom context that might be attached to this instance.
    ///
    /// Note that it could not be named `context()` as this method already exists.
    fn custom_context(&self) -> Option<Context>;

    /// Return our custom context or default it to the root-cause of the error.
    fn custom_context_or_root_cause(&self) -> Context;
}

impl private::Sealed for anyhow::Error {}
impl AnyhowContextExt for anyhow::Error {
    fn custom_context(&self) -> Option<Context> {
        if let Some(ctx) = self.downcast_ref::<Context>() {
            Some(ctx.clone())
        } else {
            self.downcast_ref::<Code>().map(|code| (*code).into())
        }
    }

    fn custom_context_or_root_cause(&self) -> Context {
        self.custom_context().unwrap_or_else(|| Context {
            code: Code::Unknown,
            message: Some(self.root_cause().to_string().into()),
        })
    }
}
