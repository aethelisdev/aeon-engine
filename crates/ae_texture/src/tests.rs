// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for texture data, procedural fallbacks, mipmap generation, path sanitization, and asset storage.

#[cfg(test)]
mod tests {
    use crate::asset::{TexturePathMap, TextureStorage};
    use crate::data::{
        ColorSpace, CpuTextureData, FilterMode, SamplerConfig, TextureMapType, WrapMode,
    };
    use crate::fallback::FallbackTextureGenerator;
    use crate::loader::is_safe_path;
    use crate::mipmap::generate_mipmap_chain;

    #[test]
    fn test_cpu_texture_data_creation() {
        let tex = CpuTextureData::new(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
            ColorSpace::Srgb,
            "test_label",
        );
        assert_eq!(tex.width, 2);
        assert_eq!(tex.height, 2);
        assert_eq!(tex.bytes.len(), 16);
        assert_eq!(tex.color_space, ColorSpace::Srgb);
        assert_eq!(tex.label, "test_label");
    }

    #[test]
    fn test_fallback_white_texture_generation() {
        let white = FallbackTextureGenerator::white_1x1();
        assert_eq!(white.width, 1);
        assert_eq!(white.height, 1);
        assert_eq!(white.bytes, vec![255, 255, 255, 255]);
        assert_eq!(white.color_space, ColorSpace::Srgb);
        assert_eq!(white.mipmaps.len(), 1);
    }

    #[test]
    fn test_fallback_flat_normal_generation() {
        let normal = FallbackTextureGenerator::flat_normal_1x1();
        assert_eq!(normal.width, 1);
        assert_eq!(normal.height, 1);
        assert_eq!(normal.bytes, vec![128, 128, 255, 255]);
        assert_eq!(normal.color_space, ColorSpace::Linear);
        assert_eq!(normal.mipmaps.len(), 1);
    }

    #[test]
    fn test_checkerboard_missing_texture_generation() {
        let checker = FallbackTextureGenerator::checkerboard_missing(4, 4, 2);
        assert_eq!(checker.width, 4);
        assert_eq!(checker.height, 4);
        assert_eq!(checker.bytes.len(), 64);
        assert_eq!(checker.mipmaps.len(), 3); // 4x4, 2x2, 1x1
    }

    #[test]
    fn test_mipmap_chain_generation_4x4_to_1x1() {
        let bytes = vec![100; 4 * 4 * 4]; // 4x4 RGBA
        let chain = generate_mipmap_chain(4, 4, &bytes);
        assert_eq!(chain.len(), 3);

        assert_eq!(chain[0].width, 4);
        assert_eq!(chain[0].height, 4);
        assert_eq!(chain[0].bytes.len(), 64);

        assert_eq!(chain[1].width, 2);
        assert_eq!(chain[1].height, 2);
        assert_eq!(chain[1].bytes.len(), 16);

        assert_eq!(chain[2].width, 1);
        assert_eq!(chain[2].height, 1);
        assert_eq!(chain[2].bytes.len(), 4);
    }

    #[test]
    fn test_texture_map_type_color_space_assignment() {
        assert_eq!(
            TextureMapType::Albedo.default_color_space(),
            ColorSpace::Srgb
        );
        assert_eq!(
            TextureMapType::Emissive.default_color_space(),
            ColorSpace::Srgb
        );
        assert_eq!(
            TextureMapType::Normal.default_color_space(),
            ColorSpace::Linear
        );
        assert_eq!(
            TextureMapType::MetallicRoughness.default_color_space(),
            ColorSpace::Linear
        );
        assert_eq!(
            TextureMapType::AmbientOcclusion.default_color_space(),
            ColorSpace::Linear
        );
    }

    #[test]
    fn test_path_security_sanitization() {
        assert!(is_safe_path("textures/albedo.png"));
        assert!(is_safe_path("C:/assets/texture.jpg"));
        assert!(!is_safe_path("../secret.png"));
        assert!(!is_safe_path("textures/../../etc/passwd"));
    }

    #[test]
    fn test_texture_storage_and_path_map() {
        let mut storage = TextureStorage::<CpuTextureData>::new();
        let mut path_map = TexturePathMap::new();

        let tex = FallbackTextureGenerator::white_1x1();
        let handle = storage.insert(tex.clone());
        let path = std::path::PathBuf::from("/canonical/white.png");

        path_map.insert(path.clone(), handle);

        assert_eq!(storage.len(), 1);
        assert_eq!(path_map.get(&path), Some(handle));

        let retrieved = storage.get(handle);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().width, 1);
    }

    #[test]
    fn test_sampler_config_defaults() {
        let sampler = SamplerConfig::default();
        assert_eq!(sampler.min_filter, FilterMode::Nearest);
        assert_eq!(sampler.mag_filter, FilterMode::Linear);
        assert_eq!(sampler.wrap_u, WrapMode::ClampToEdge);
        assert_eq!(sampler.wrap_v, WrapMode::ClampToEdge);
    }
}