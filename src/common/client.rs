use crate::app::{
    constant::{
        CONTENT_TYPE_CONNECT_PROTO, CURSOR_API2_HOST, CURSOR_HOST, CURSOR_SETTINGS_URL,
        HEADER_NAME_GHOST_MODE, TRUE,
    },
    lazy::{
        CURSOR_API2_CHAT_URL, CURSOR_API2_STRIPE_URL, CURSOR_USAGE_API_URL, CURSOR_USER_API_URL,
        REVERSE_PROXY_HOST, USE_PROXY,
    },
};
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, CONTENT_TYPE, COOKIE, DNT,
    HOST, ORIGIN, PRAGMA, REFERER, TE, TRANSFER_ENCODING, USER_AGENT,
};
use reqwest::{Client, RequestBuilder};
use uuid::Uuid;

macro_rules! def_const {
    ($name:ident, $value:expr) => {
        const $name: &'static str = $value;
    };
}

def_const!(SEC_FETCH_DEST, "sec-fetch-dest");
def_const!(SEC_FETCH_MODE, "sec-fetch-mode");
def_const!(SEC_FETCH_SITE, "sec-fetch-site");
def_const!(SEC_GPC, "sec-gpc");
def_const!(PRIORITY, "priority");

def_const!(ONE, "1");
def_const!(ENCODINGS, "gzip,br");
def_const!(VALUE_ACCEPT, "*/*");
def_const!(VALUE_LANGUAGE, "zh-CN");
def_const!(EMPTY, "empty");
def_const!(CORS, "cors");
def_const!(NO_CACHE, "no-cache");
def_const!(UA_WIN, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36");
def_const!(SAME_ORIGIN, "same-origin");
def_const!(KEEP_ALIVE, "keep-alive");
def_const!(TRAILERS, "trailers");
def_const!(U_EQ_4, "u=4");

def_const!(PROXY_HOST, "x-co");

/// 返回预构建的 Cursor API 客户端
///
/// # 参数
///
/// * `auth_token` - 授权令牌
/// * `checksum` - 校验和
/// * `endpoint` - API 端点路径
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_client(auth_token: &str, checksum: &str) -> RequestBuilder {
    let trace_id = Uuid::new_v4().to_string();

    let client = if *USE_PROXY {
        Client::new()
            .post(&*CURSOR_API2_CHAT_URL)
            .header(HOST, &*REVERSE_PROXY_HOST)
            .header(PROXY_HOST, CURSOR_API2_HOST)
    } else {
        Client::new()
            .post(&*CURSOR_API2_CHAT_URL)
            .header(HOST, CURSOR_API2_HOST)
    };

    client
        .header(CONTENT_TYPE, CONTENT_TYPE_CONNECT_PROTO)
        .bearer_auth(auth_token)
        .header("connect-accept-encoding", ENCODINGS)
        .header("connect-protocol-version", ONE)
        .header(USER_AGENT, "connect-es/1.6.1")
        .header("x-amzn-trace-id", format!("Root={}", trace_id))
        // .header("x-client-key", client_key)
        .header("x-cursor-checksum", checksum)
        .header("x-cursor-client-version", "0.42.5")
        .header("x-cursor-timezone", "Asia/Shanghai")
        .header(HEADER_NAME_GHOST_MODE, TRUE)
        .header("x-request-id", trace_id)
        .header(CONNECTION, KEEP_ALIVE)
        .header(TRANSFER_ENCODING, "chunked")
}

/// 返回预构建的获取 Stripe 账户信息的 Cursor API 客户端
///
/// # 参数
///
/// * `auth_token` - 授权令牌
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_profile_client(auth_token: &str) -> RequestBuilder {
    let client = if *USE_PROXY {
        Client::new()
            .get(&*CURSOR_API2_STRIPE_URL)
            .header(HOST, &*REVERSE_PROXY_HOST)
            .header(PROXY_HOST, CURSOR_API2_HOST)
    } else {
        Client::new()
            .get(&*CURSOR_API2_STRIPE_URL)
            .header(HOST, CURSOR_API2_HOST)
    };

    client
        .header("sec-ch-ua", "\"Not-A.Brand\";v=\"99\", \"Chromium\";v=\"124\"")
        .header(HEADER_NAME_GHOST_MODE, TRUE)
        .header("sec-ch-ua-mobile", "?0")
        .bearer_auth(auth_token)
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.42.5 Chrome/124.0.6367.243 Electron/30.4.0 Safari/537.36",
        )
        .header("sec-ch-ua-platform", "\"Windows\"")
        .header(ACCEPT, VALUE_ACCEPT)
        .header(ORIGIN, "vscode-file://vscode-app")
        .header(SEC_FETCH_SITE, "cross-site")
        .header(SEC_FETCH_MODE, CORS)
        .header(SEC_FETCH_DEST, EMPTY)
        .header(ACCEPT_ENCODING, ENCODINGS)
        .header(ACCEPT_LANGUAGE, VALUE_LANGUAGE)
        .header(PRIORITY, "u=1, i")
}

/// 返回预构建的获取使用情况的 Cursor API 客户端
///
/// # 参数
///
/// * `user_id` - 用户 ID
/// * `auth_token` - 授权令牌
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_usage_client(user_id: &str, auth_token: &str) -> RequestBuilder {
    let session_token = format!("{}%3A%3A{}", user_id, auth_token);

    let client = if *USE_PROXY {
        Client::new()
            .get(&*CURSOR_USAGE_API_URL)
            .header(HOST, &*REVERSE_PROXY_HOST)
            .header(PROXY_HOST, CURSOR_HOST)
    } else {
        Client::new()
            .get(&*CURSOR_USAGE_API_URL)
            .header(HOST, CURSOR_HOST)
    };

    client
        .header(USER_AGENT, UA_WIN)
        .header(ACCEPT, VALUE_ACCEPT)
        .header(ACCEPT_LANGUAGE, VALUE_LANGUAGE)
        .header(ACCEPT_ENCODING, ENCODINGS)
        .header(REFERER, CURSOR_SETTINGS_URL)
        .header(DNT, ONE)
        .header(SEC_GPC, ONE)
        .header(SEC_FETCH_DEST, EMPTY)
        .header(SEC_FETCH_MODE, CORS)
        .header(SEC_FETCH_SITE, SAME_ORIGIN)
        .header(CONNECTION, KEEP_ALIVE)
        .header(PRAGMA, NO_CACHE)
        .header(CACHE_CONTROL, NO_CACHE)
        .header(TE, TRAILERS)
        .header(PRIORITY, U_EQ_4)
        .header(
            COOKIE,
            &format!("WorkosCursorSessionToken={}", session_token),
        )
        .query(&[("user", user_id)])
}

/// 返回预构建的获取用户信息的 Cursor API 客户端
///
/// # 参数
///
/// * `user_id` - 用户 ID
/// * `auth_token` - 授权令牌
///
/// # 返回
///
/// * `reqwest::RequestBuilder` - 配置好的请求构建器
pub fn build_userinfo_client(user_id: &str, auth_token: &str) -> RequestBuilder {
    let session_token = format!("{}%3A%3A{}", user_id, auth_token);

    let client = if *USE_PROXY {
        Client::new()
            .get(&*CURSOR_USER_API_URL)
            .header(HOST, &*REVERSE_PROXY_HOST)
            .header(PROXY_HOST, CURSOR_HOST)
    } else {
        Client::new()
            .get(&*CURSOR_USER_API_URL)
            .header(HOST, CURSOR_HOST)
    };

    client
        .header(USER_AGENT, UA_WIN)
        .header(ACCEPT, VALUE_ACCEPT)
        .header(ACCEPT_LANGUAGE, VALUE_LANGUAGE)
        .header(ACCEPT_ENCODING, ENCODINGS)
        .header(REFERER, CURSOR_SETTINGS_URL)
        .header(DNT, ONE)
        .header(SEC_GPC, ONE)
        .header(SEC_FETCH_DEST, EMPTY)
        .header(SEC_FETCH_MODE, CORS)
        .header(SEC_FETCH_SITE, SAME_ORIGIN)
        .header(CONNECTION, KEEP_ALIVE)
        .header(PRAGMA, NO_CACHE)
        .header(CACHE_CONTROL, NO_CACHE)
        .header(TE, TRAILERS)
        .header(PRIORITY, U_EQ_4)
        .header(
            COOKIE,
            &format!("WorkosCursorSessionToken={}", session_token),
        )
        .query(&[("user", user_id)])
}
