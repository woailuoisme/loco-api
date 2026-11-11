use loco_api::{app::App, setup::storage};
use loco_rs::testing::prelude::*;
use serial_test::serial;
use std::path::Path;

#[tokio::test]
#[serial]
async fn test_storage_init_in_test_environment() {
    let boot = boot_test::<App>().await.unwrap();
    let ctx = storage::init(boot.app_context).await.unwrap();

    // 校验测试环境下成功初始化内存存储驱动
    assert!(matches!(ctx.environment, loco_rs::environment::Environment::Test));

    // 验证内存存储驱动实际具备读写能力
    let test_path = Path::new("test_isolation.txt");
    let content = bytes::Bytes::from_static(b"loco-test-inmem");
    ctx.storage.upload(test_path, &content).await.unwrap();
    assert!(ctx.storage.exists(test_path).await.unwrap());
    let downloaded: Vec<u8> = ctx.storage.download(test_path).await.unwrap();
    assert_eq!(downloaded, content.as_ref());
}
