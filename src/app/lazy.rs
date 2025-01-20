use crate::{
    app::constant::{
        CURSOR_API2_HOST, CURSOR_HOST, DEFAULT_TOKEN_FILE_NAME, DEFAULT_TOKEN_LIST_FILE_NAME,
        EMPTY_STRING,
    },
    common::utils::parse_string_from_env,
};
use std::sync::LazyLock;

macro_rules! def_pub_static {
    // 基础版本：直接存储 String
    ($name:ident, $value:expr) => {
        pub static $name: LazyLock<String> = LazyLock::new(|| $value);
    };

    // 环境变量版本
    ($name:ident, env: $env_key:expr, default: $default:expr) => {
        pub static $name: LazyLock<String> =
            LazyLock::new(|| parse_string_from_env($env_key, $default).trim().to_string());
    };
}

// macro_rules! def_pub_static_getter {
//     ($name:ident) => {
//         paste::paste! {
//             pub fn [<get_ $name:lower>]() -> String {
//                 (*$name).clone()
//             }
//         }
//     };
// }

def_pub_static!(ROUTE_PREFIX, env: "ROUTE_PREFIX", default: EMPTY_STRING);
def_pub_static!(AUTH_TOKEN, env: "AUTH_TOKEN", default: EMPTY_STRING);
def_pub_static!(TOKEN_FILE, env: "TOKEN_FILE", default: DEFAULT_TOKEN_FILE_NAME);
def_pub_static!(TOKEN_LIST_FILE, env: "TOKEN_LIST_FILE", default: DEFAULT_TOKEN_LIST_FILE_NAME);
def_pub_static!(ROUTE_MODELS_PATH, format!("{}/v1/models", *ROUTE_PREFIX));
def_pub_static!(
    ROUTE_CHAT_PATH,
    format!("{}/v1/chat/completions", *ROUTE_PREFIX)
);

pub static START_TIME: LazyLock<chrono::DateTime<chrono::Local>> =
    LazyLock::new(chrono::Local::now);

pub fn get_start_time() -> chrono::DateTime<chrono::Local> {
    *START_TIME
}

def_pub_static!(DEFAULT_INSTRUCTIONS, env: "DEFAULT_INSTRUCTIONS", default: "Respond in Chinese by default");

def_pub_static!(REVERSE_PROXY_HOST, env: "REVERSE_PROXY_HOST", default: "");

pub static USE_PROXY: LazyLock<bool> = LazyLock::new(|| !REVERSE_PROXY_HOST.is_empty());

pub static CURSOR_API2_CHAT_URL: LazyLock<String> = LazyLock::new(|| {
    let host = if *USE_PROXY {
        &*REVERSE_PROXY_HOST
    } else {
        CURSOR_API2_HOST
    };
    format!("https://{}/aiserver.v1.AiService/StreamChat", host)
});

pub static CURSOR_API2_STRIPE_URL: LazyLock<String> = LazyLock::new(|| {
    let host = if *USE_PROXY {
        &*REVERSE_PROXY_HOST
    } else {
        CURSOR_API2_HOST
    };
    format!("https://{}/auth/full_stripe_profile", host)
});

pub static CURSOR_USAGE_API_URL: LazyLock<String> = LazyLock::new(|| {
    let host = if *USE_PROXY {
        &*REVERSE_PROXY_HOST
    } else {
        CURSOR_HOST
    };
    format!("https://{}/api/usage", host)
});

pub static CURSOR_USER_API_URL: LazyLock<String> = LazyLock::new(|| {
    let host = if *USE_PROXY {
        &*REVERSE_PROXY_HOST
    } else {
        CURSOR_HOST
    };
    format!("https://{}/api/auth/me", host)
});

// pub static DEBUG: LazyLock<bool> = LazyLock::new(|| parse_bool_from_env("DEBUG", false));

// #[macro_export]
// macro_rules! debug_println {
//     ($($arg:tt)*) => {
//         if *crate::app::statics::DEBUG {
//             println!($($arg)*);
//         }
//     };
// }
