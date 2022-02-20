// Copyright 2022 The Engula Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use thiserror::Error;
use tonic::{Code, Status};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0} is not found")]
    NotFound(String),
    #[error("{0} already exists")]
    AlreadyExists(String),
    #[error("invalid argument")]
    InvalidArgument,
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        let (code, message) = match err {
            Error::NotFound(m) => (Code::NotFound, m),
            Error::AlreadyExists(m) => (Code::AlreadyExists, m),
            Error::InvalidArgument => (Code::InvalidArgument, "".to_owned()),
        };
        Self::new(code, message)
    }
}

impl From<engula_cooperator::Error> for Error {
    fn from(err: engula_cooperator::Error) -> Self {
        match err {
            engula_cooperator::Error::NotFound(m) => Self::NotFound(m),
            engula_cooperator::Error::AlreadyExists(m) => Self::AlreadyExists(m),
            engula_cooperator::Error::InvalidArgument => Self::InvalidArgument,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;