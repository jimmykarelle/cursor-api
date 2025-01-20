macro_rules! def_pub_const {
    ($name:ident, $value:expr) => {
        pub const $name: &'static str = $value;
    };
}

def_pub_const!(PKG_VERSION, env!("CARGO_PKG_VERSION"));
// def_pub_const!(PKG_NAME, env!("CARGO_PKG_NAME"));
// def_pub_const!(PKG_DESCRIPTION, env!("CARGO_PKG_DESCRIPTION"));
// def_pub_const!(PKG_AUTHORS, env!("CARGO_PKG_AUTHORS"));
// def_pub_const!(PKG_REPOSITORY, env!("CARGO_PKG_REPOSITORY"));

def_pub_const!(EMPTY_STRING, "");

def_pub_const!(ROUTE_ROOT_PATH, "/");
def_pub_const!(ROUTE_HEALTH_PATH, "/health");
def_pub_const!(ROUTE_GET_CHECKSUM, "/get-checksum");
def_pub_const!(ROUTE_GET_USER_INFO_PATH, "/get-userinfo");
def_pub_const!(ROUTE_API_PATH, "/api");
def_pub_const!(ROUTE_LOGS_PATH, "/logs");
def_pub_const!(ROUTE_CONFIG_PATH, "/config");
def_pub_const!(ROUTE_TOKENINFO_PATH, "/tokeninfo");
def_pub_const!(ROUTE_GET_TOKENINFO_PATH, "/get-tokeninfo");
def_pub_const!(ROUTE_UPDATE_TOKENINFO_PATH, "/update-tokeninfo");
def_pub_const!(ROUTE_ENV_EXAMPLE_PATH, "/env-example");
def_pub_const!(ROUTE_STATIC_PATH, "/static/:path");
def_pub_const!(ROUTE_SHARED_STYLES_PATH, "/static/shared-styles.css");
def_pub_const!(ROUTE_SHARED_JS_PATH, "/static/shared.js");
def_pub_const!(ROUTE_ABOUT_PATH, "/about");
def_pub_const!(ROUTE_README_PATH, "/readme");
def_pub_const!(ROUTE_BASIC_CALIBRATION_PATH, "/basic-calibration");

def_pub_const!(DEFAULT_TOKEN_FILE_NAME, ".token");
def_pub_const!(DEFAULT_TOKEN_LIST_FILE_NAME, ".token-list");

def_pub_const!(STATUS_PENDING, "pending");
def_pub_const!(STATUS_SUCCESS, "success");
def_pub_const!(STATUS_FAILED, "failed");

def_pub_const!(HEADER_NAME_GHOST_MODE, "x-ghost-mode");

def_pub_const!(TRUE, "true");
def_pub_const!(FALSE, "false");

// def_pub_const!(CONTENT_TYPE_PROTO, "application/proto");
def_pub_const!(CONTENT_TYPE_CONNECT_PROTO, "application/connect+proto");
def_pub_const!(CONTENT_TYPE_TEXT_HTML_WITH_UTF8, "text/html;charset=utf-8");
def_pub_const!(CONTENT_TYPE_TEXT_PLAIN_WITH_UTF8, "text/plain;charset=utf-8");
def_pub_const!(CONTENT_TYPE_TEXT_CSS_WITH_UTF8, "text/css;charset=utf-8");
def_pub_const!(CONTENT_TYPE_TEXT_JS_WITH_UTF8, "text/javascript;charset=utf-8");

def_pub_const!(AUTHORIZATION_BEARER_PREFIX, "Bearer ");

def_pub_const!(CURSOR_API2_HOST, "api2.cursor.sh");
def_pub_const!(CURSOR_HOST, "www.cursor.com");
def_pub_const!(CURSOR_SETTINGS_URL, "https://www.cursor.com/settings");

def_pub_const!(OBJECT_CHAT_COMPLETION, "chat.completion");
def_pub_const!(OBJECT_CHAT_COMPLETION_CHUNK, "chat.completion.chunk");

// def_pub_const!(CURSOR_API2_STREAM_CHAT, "StreamChat");
// def_pub_const!(CURSOR_API2_GET_USER_INFO, "GetUserInfo");

def_pub_const!(FINISH_REASON_STOP, "stop");

def_pub_const!(ERR_UPDATE_CONFIG, "无法更新配置");
def_pub_const!(ERR_RESET_CONFIG, "无法重置配置");
def_pub_const!(ERR_INVALID_PATH, "无效的路径");

// def_pub_const!(ERR_CHECKSUM_NO_GOOD, "checksum no good");
