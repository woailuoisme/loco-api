use loco_api::app::App;
use loco_rs::testing::prelude::*;

#[tokio::test]
#[serial_test::serial]
async fn can_get_file_not_found() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/v1/files/uploads/non_existent.txt").await;
        res.assert_status(axum::http::StatusCode::NOT_FOUND);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn file_upload_requires_authentication() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.post("/v1/files").await;
        res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
    })
    .await;
}

#[tokio::test]
#[serial_test::serial]
async fn can_upload_image_and_receive_variants() {
    use axum_test::multipart::{MultipartForm, Part};

    request::<App, _, _>(|request, ctx| async move {
        let auth = super::prepare_data::init_user_login_unique(&request, &ctx).await;

        let form = MultipartForm::new().add_part(
            "file",
            Part::bytes(b"fake_png_binary_data".to_vec())
                .file_name("sample_photo.png")
                .mime_type("image/png"),
        );

        let res = request
            .post("/v1/files")
            .add_header(axum::http::header::AUTHORIZATION, format!("Bearer {}", auth.token))
            .multipart(form)
            .await;

        assert_eq!(res.status_code(), axum::http::StatusCode::OK);
        let list: Vec<loco_api::dtos::files::FileUploadResource> = serde_json::from_str(&res.text()).unwrap();
        assert_eq!(list.len(), 1);

        let item = &list[0];
        assert_eq!(item.filename, "sample_photo.png");
        assert!(item.key.starts_with("uploads/"));

        // 验证 variants 是否包含多端优化链接
        assert!(item.variants.is_some());
        let variants = item.variants.as_ref().unwrap();
        assert_eq!(variants.path, item.key);
        assert!(!variants.url.is_empty());
        assert!(!variants.webp.is_empty());
        assert!(!variants.thumb.is_empty());
        assert!(!variants.cover.is_empty());
    })
    .await;
}
