// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Iris registration identifier for the renderer-owned resolved viewport image.

use irisui::prelude::ExternalTextureId;

/// Stable identity reserved for the editor's resolved 3D viewport image.
pub const VIEWPORT_TEXTURE_ID: ExternalTextureId = ExternalTextureId(0);