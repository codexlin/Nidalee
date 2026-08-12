// HTTP客户端管理模块 - 统一管理HTTP客户端的创建和配置
use once_cell::sync::Lazy;
use reqwest::Client;

/// 全局的LCU HTTP客户端，单例模式
static LCU_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create LCU HTTP client")
});

/// 获取配置好的LCU HTTP客户端
pub fn get_lcu_client() -> &'static Client {
    &LCU_CLIENT
}

/// 获取配置好的公共HTTP客户端（用于访问Riot公开API等）
pub fn get_public_client() -> &'static Client {
    static PUBLIC_CLIENT: Lazy<Client> = Lazy::new(|| {
        Client::builder()
            .https_only(true)
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Nidalee/1.0")
            .build()
            .expect("Failed to create public HTTP client")
    });
    &PUBLIC_CLIENT
}
