use loco_api::common::media::{
    AliyunOssDriver, CloudflareDriver, DirectDriver, ImageDriver, ImageOptions, ImagePreset, ImgproxyDriver, Media,
    MediaConfig, MediaDriverKind, ResizeFit,
};

#[test]
fn test_direct_driver_urls() {
    let driver = DirectDriver::new("https://static.example.com/files/");
    let path = "/uploads/2026/11/avatar.jpg";

    let variants = driver.build_variants(path);
    assert_eq!(variants.path, "uploads/2026/11/avatar.jpg");
    assert_eq!(
        variants.url,
        "https://static.example.com/files/uploads/2026/11/avatar.jpg"
    );
    assert_eq!(
        variants.webp,
        "https://static.example.com/files/uploads/2026/11/avatar.jpg"
    );
    assert_eq!(
        variants.thumb,
        "https://static.example.com/files/uploads/2026/11/avatar.jpg"
    );
    assert_eq!(
        variants.cover,
        "https://static.example.com/files/uploads/2026/11/avatar.jpg"
    );

    // 测试未设置 base_url 的情况
    let bare_driver = DirectDriver::default();
    assert_eq!(
        bare_driver.build_url("avatar.png", &ImageOptions::default()),
        "/avatar.png"
    );
}

#[test]
fn test_cloudflare_driver_urls() {
    let driver = CloudflareDriver::new("https://cdn.example.com/");
    let path = "uploads/cover.jpg";

    // 1. 原图直链
    let original = driver.url_with_preset(path, ImagePreset::Original);
    assert_eq!(original, "https://cdn.example.com/uploads/cover.jpg");

    // 2. WebP 优化
    let webp = driver.url_with_preset(path, ImagePreset::Webp);
    assert_eq!(
        webp,
        "https://cdn.example.com/cdn-cgi/image/format=webp,quality=85/uploads/cover.jpg"
    );

    // 3. Thumb 缩略图 (150x150 cover)
    let thumb = driver.url_with_preset(path, ImagePreset::Thumb);
    assert_eq!(
        thumb,
        "https://cdn.example.com/cdn-cgi/image/width=150,height=150,fit=cover,format=webp,quality=80/uploads/cover.jpg"
    );

    // 4. Cover 封面图 (800x450 cover)
    let cover = driver.url_with_preset(path, ImagePreset::Cover);
    assert_eq!(
        cover,
        "https://cdn.example.com/cdn-cgi/image/width=800,height=450,fit=cover,format=webp,quality=85/uploads/cover.jpg"
    );

    // 5. 自定义动态选项
    let custom = driver.build_url(
        path,
        &ImageOptions::new()
            .with_size(300, 200)
            .with_fit(ResizeFit::Contain)
            .with_format("avif")
            .with_quality(90),
    );
    assert_eq!(
        custom,
        "https://cdn.example.com/cdn-cgi/image/width=300,height=200,fit=contain,format=avif,quality=90/uploads/cover.jpg"
    );
}

#[test]
fn test_aliyun_oss_driver_urls() {
    let driver = AliyunOssDriver::new("my-bucket", "https://oss-cn-hangzhou.aliyuncs.com/");
    let path = "uploads/product.png";

    // 1. 原图直链
    let original = driver.url_with_preset(path, ImagePreset::Original);
    assert_eq!(
        original,
        "https://my-bucket.oss-cn-hangzhou.aliyuncs.com/uploads/product.png"
    );

    // 2. WebP 优化
    let webp = driver.url_with_preset(path, ImagePreset::Webp);
    assert_eq!(
        webp,
        "https://my-bucket.oss-cn-hangzhou.aliyuncs.com/uploads/product.png?x-oss-process=image/format,webp/quality,q_85"
    );

    // 3. Thumb 缩略图
    let thumb = driver.url_with_preset(path, ImagePreset::Thumb);
    assert_eq!(
        thumb,
        "https://my-bucket.oss-cn-hangzhou.aliyuncs.com/uploads/product.png?x-oss-process=image/resize,m_fill,w_150,h_150/format,webp/quality,q_80"
    );

    // 4. Cover 封面图
    let cover = driver.url_with_preset(path, ImagePreset::Cover);
    assert_eq!(
        cover,
        "https://my-bucket.oss-cn-hangzhou.aliyuncs.com/uploads/product.png?x-oss-process=image/resize,m_fill,w_800,h_450/format,webp/quality,q_85"
    );
}

#[test]
fn test_imgproxy_driver_unsafe_mode() {
    let driver = ImgproxyDriver::new(
        "http://imgproxy.local:8080",
        None,
        None,
        Some("http://files.local:5150".to_string()),
    );
    let path = "uploads/banner.jpg";

    let thumb = driver.url_with_preset(path, ImagePreset::Thumb);
    assert!(thumb.starts_with("http://imgproxy.local:8080/unsafe/"));
    assert!(thumb.contains("rs:fill:150:150"));
    assert!(thumb.contains("ext:webp"));
    assert!(thumb.contains("/plain/http://files.local:5150/uploads/banner.jpg"));
}

#[test]
fn test_imgproxy_driver_signed_mode() {
    // 32-byte (64 hex characters) key & salt
    let key_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let salt_hex = "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f";

    let driver = ImgproxyDriver::new(
        "https://images.example.com",
        Some(key_hex),
        Some(salt_hex),
        Some("s3://media-bucket".to_string()),
    );
    let path = "photo.jpg";

    let url = driver.url_with_preset(path, ImagePreset::Thumb);
    assert!(!url.contains("/unsafe/"));
    assert!(url.starts_with("https://images.example.com/"));
    assert!(url.contains("/rs:fill:150:150/q:80/ext:webp/plain/s3://media-bucket/photo.jpg"));

    // 验证不同路径与预设生成的签名各不相同且确定性一致
    let url2 = driver.url_with_preset(path, ImagePreset::Thumb);
    assert_eq!(url, url2);

    let url_cover = driver.url_with_preset(path, ImagePreset::Cover);
    assert_ne!(url, url_cover);
}

#[test]
fn test_media_config_and_facade() {
    let config = MediaConfig {
        driver: MediaDriverKind::Direct,
        base_url: Some("https://cdn.my-site.com".to_string()),
        ..Default::default()
    };

    let variants = Media::variants(&config, "images/logo.png");
    assert_eq!(variants.path, "images/logo.png");
    assert_eq!(variants.url, "https://cdn.my-site.com/images/logo.png");
    assert_eq!(variants.thumb, "https://cdn.my-site.com/images/logo.png");
}
