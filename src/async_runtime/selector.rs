// This file is part of gui-tools.
//
// gui-tools is free software: you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option)
// any later version.
//
// gui-tools is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty
// of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General
// Public License along with gui-tools. If not, see
// <https://www.gnu.org/licenses/>.

use super::GeneralRuntime;
use crate::Result;
use alloc::{boxed::Box, vec, vec::Vec};
use core::{future::Future, pin::Pin};

impl RuntimeBuilder {
    /// Creates a new `RuntimeBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Runtime` using the information encoded in this
    /// structure.
    pub fn finish(&self) -> impl Future<Output = Result<GeneralRuntime>> + Send + '_ {
        backend_list().find_backend(self)
    }
}

/// Select the runtime to use.
pub async fn select_runtime() -> Result<GeneralRuntime> {
    RuntimeBuilder::new().finish().await
}

/// Add a new entry to the backend list.
pub fn add_backend(backend: Backend) {
    backend_list().add(backend);
}

fn backend_list() -> &'static BackendList {
    static BACKEND_LIST: once_cell::sync::OnceCell<BackendList> = once_cell::sync::OnceCell::new();

    BACKEND_LIST.get_or_init(|| {
        // TODO: list of regular backends here
        let backends = vec![];
        BackendList::new(backends)
    })
}

macro_rules! decl_backend_info {
    (
        $(
            $(#[$attr: meta])*
            $fname: ident : $fty: ty
        ),*
    ) => {
        /// Information about a backend's operation.
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct BackendInfo {
            $($(#[$attr])* pub $fname: $fty),*
        }

        /// A builder used to select the runtime to use.
        #[derive(Debug, Clone, Default)]
        pub struct RuntimeBuilder {
            $($fname: Option<$fty>),*
        }

        impl RuntimeBuilder {
            $(
                $(#[$attr])*
                pub fn $fname(&mut self, $fname: $fty) -> &mut Self {
                    self.$fname = Some($fname);
                    self
                }
            )*

            /// Tell if this builder matches the given backend info.
            fn matches(&self, backend: &BackendInfo) -> bool {
                $(
                    if let Some(ref value) = self.$fname {
                        if &backend.$fname != value {
                            return false;
                        }
                    }
                )*
                true
            }
        }
    }
}

decl_backend_info! {
    /// The name of the backend
    name: &'static str,
    /// Whether or not the backend supports hardware acceleration
    /// of any kind.
    hardware_accelerated: bool
}

/// A backend to be inserted into the backend list.
#[derive(Copy, Clone)]
pub struct Backend {
    /// Information about this backend.
    pub info: BackendInfo,
    /// A function that produces the backend.
    pub producer: Producer,
}

type Producer = fn(&RuntimeBuilder) -> Pin<Box<dyn Future<Output = Result<GeneralRuntime>> + Send + 'static>>;

/// A list of backends to be used by the runtime.
struct BackendList {
    list: std::sync::RwLock<Vec<Backend>>,
}

impl BackendList {
    fn new(backends: Vec<Backend>) -> Self {
                Self {
                    list: std::sync::RwLock::new(backends)
                }
    }

    fn add(&self, backend: Backend) {
                let mut guard = match self.list.write() {
                    Ok(guard) => guard,
                    Err(err) => {
                        tracing::error!("Failed to acquire write-lock: {:?}", &err);
                        err.into_inner()
                    }
                };

                guard.push(backend);
    }

    fn copy_out_list(&self) -> Vec<Backend> {
        let guard = match self.list.read() {
            Ok(guard) => guard,
            Err(err) => {
                tracing::error!("Failed to acquire read-lock: {:?}", &err);
                err.into_inner()
            }
        };

        // copy out the entire list
        guard.clone()
    }

    async fn find_backend(&self, builder: &RuntimeBuilder) -> Result<GeneralRuntime> {
        let list = self.copy_out_list();

        let mut last_error = None;
        for backend in list {
            // try to instantiate the backend, if it matches
            if builder.matches(&backend.info) {
                match (backend.producer)(builder).await {
                    Ok(runtime) => return Ok(runtime),
                    Err(e) => {
                        tracing::warn!("Could not generate backend {}: {}", backend.info.name, &e);
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| todo!()))
    }
}
