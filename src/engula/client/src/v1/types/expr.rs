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

use engula_apis::v1::*;

pub struct SelectExpr(TypedExpr);

impl From<TypedExpr> for SelectExpr {
    fn from(v: TypedExpr) -> Self {
        Self(v)
    }
}

impl From<SelectExpr> for TypedExpr {
    fn from(v: SelectExpr) -> Self {
        v.0
    }
}

pub struct MutateExpr(TypedExpr);

impl From<TypedExpr> for MutateExpr {
    fn from(v: TypedExpr) -> Self {
        Self(v)
    }
}

impl From<MutateExpr> for TypedExpr {
    fn from(v: MutateExpr) -> Self {
        v.0
    }
}