// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Abstraction of signed URL generation for those object store implementations that support it

use crate::{Result, path::Path};
use async_trait::async_trait;
use reqwest::Method;
use std::{fmt, time::Duration};
use url::Url;

/// Options customizing a signed URL beyond method, path, and lifetime.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SignedUrlOptions {
    /// Override for the `Content-Disposition` header served with the response.
    ///
    /// The override is part of the signature (`response-content-disposition`
    /// for AWS SigV4, `rscd` for Azure SAS), so holders of the URL cannot
    /// alter it.
    pub response_content_disposition: Option<String>,
}

impl SignedUrlOptions {
    /// Returns true when these options request no customization.
    pub fn is_default(&self) -> bool {
        self.response_content_disposition.is_none()
    }
}

/// Universal API to generate presigned URLs from multiple object store services.
#[async_trait]
pub trait Signer: Send + Sync + fmt::Debug + 'static {
    /// Given the intended [`Method`] and [`Path`] to use and the desired length of time for which
    /// the URL should be valid, return a signed [`Url`] created with the object store
    /// implementation's credentials such that the URL can be handed to something that doesn't have
    /// access to the object store's credentials, to allow limited access to the object store.
    async fn signed_url(&self, method: Method, path: &Path, expires_in: Duration) -> Result<Url>;

    /// [`Signer::signed_url`] with additional [`SignedUrlOptions`].
    ///
    /// The default implementation only supports default options and returns
    /// [`Error::NotImplemented`](crate::Error::NotImplemented) otherwise, so a
    /// store without support never silently drops a requested override.
    async fn signed_url_with_options(
        &self,
        method: Method,
        path: &Path,
        expires_in: Duration,
        options: SignedUrlOptions,
    ) -> Result<Url> {
        if options.is_default() {
            self.signed_url(method, path, expires_in).await
        } else {
            Err(crate::Error::NotImplemented)
        }
    }

    /// Generate signed urls for multiple paths.
    ///
    /// See [`Signer::signed_url`] for more details.
    async fn signed_urls(
        &self,
        method: Method,
        paths: &[Path],
        expires_in: Duration,
    ) -> Result<Vec<Url>> {
        let mut urls = Vec::with_capacity(paths.len());
        for path in paths {
            urls.push(self.signed_url(method.clone(), path, expires_in).await?);
        }
        Ok(urls)
    }
}
