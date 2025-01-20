use crate::{
    app::{
        constant::{
            AUTHORIZATION_BEARER_PREFIX, FINISH_REASON_STOP,
            OBJECT_CHAT_COMPLETION, OBJECT_CHAT_COMPLETION_CHUNK, STATUS_FAILED, STATUS_PENDING,
            STATUS_SUCCESS,
        },
        lazy::AUTH_TOKEN,
        model::{AppConfig, AppState, ChatRequest, RequestLog, TimingInfo, TokenInfo},
    },
    chat::{
        constant::{AVAILABLE_MODELS, USAGE_CHECK_MODELS},
        error::StreamError,
        model::{
            ChatResponse, Choice, Delta, Message, MessageContent, ModelsResponse, Role, Usage,
        },
        stream::{parse_stream_data, StreamMessage},
    },
    common::{
        client::build_client,
        models::{error::ChatError, userinfo::MembershipType, ErrorResponse},
        utils::{
            format_time_ms, get_token_profile, validate_token_and_checksum,
        },
    },
};
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderMap, StatusCode,
    },
    response::Response,
    Json,
};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::{
    convert::Infallible,
    sync::{atomic::AtomicBool, Arc},
};
use std::{
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::sync::Mutex;
use uuid::Uuid;

const REQUEST_LOGS_LIMIT: usize = 1000;

// 模型列表处理
pub async fn handle_models() -> Json<ModelsResponse> {
    Json(ModelsResponse {
        object: "list",
        data: &AVAILABLE_MODELS,
    })
}

// 聊天处理函数的签名
pub async fn handle_chat(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<ChatRequest>,
) -> Result<Response<Body>, (StatusCode, Json<ErrorResponse>)> {
    let allow_claude = AppConfig::get_allow_claude();
    // 验证模型是否支持并获取模型信息
    let model = AVAILABLE_MODELS.iter().find(|m| m.id == request.model);
    let model_supported = model.is_some();

    if !(model_supported || allow_claude && request.model.starts_with("claude")) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ChatError::ModelNotSupported(request.model).to_json()),
        ));
    }

    let request_time = chrono::Local::now();

    // 验证请求
    if request.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ChatError::EmptyMessages.to_json()),
        ));
    }

    // 获取并处理认证令牌
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix(AUTHORIZATION_BEARER_PREFIX))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        ))?;

    // 验证 AuthToken 和 获取 token 信息
    let (auth_token, checksum) = if auth_header == AUTH_TOKEN.as_str() {
        // 如果是管理员Token,使用原有逻辑
        static CURRENT_KEY_INDEX: AtomicUsize = AtomicUsize::new(0);
        let state_guard = state.lock().await;
        let token_infos = &state_guard.token_infos;

        if token_infos.is_empty() {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ChatError::NoTokens.to_json()),
            ));
        }

        let index = CURRENT_KEY_INDEX.fetch_add(1, Ordering::SeqCst) % token_infos.len();
        let token_info = &token_infos[index];
        (token_info.token.clone(), token_info.checksum.clone())
    } else {
        // 否则尝试解析token
        validate_token_and_checksum(auth_header).ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        ))?
    };

    let current_id: u64;

    // 更新请求日志
    {
        let state_clone = state.clone();
        let mut state = state.lock().await;
        state.total_requests += 1;
        state.active_requests += 1;

        // 查找最新的相同token的日志,检查使用情况
        let need_profile_check = state
            .request_logs
            .iter()
            .rev()
            .find(|log| log.token_info.token == auth_token && log.token_info.profile.is_some())
            .and_then(|log| log.token_info.profile.as_ref())
            .map(|profile| {
                if profile.stripe.membership_type != MembershipType::Free {
                    return false;
                }

                let is_premium = USAGE_CHECK_MODELS.contains(&request.model.as_str());
                let standard = &profile.usage.standard;
                let premium = &profile.usage.premium;

                if is_premium {
                    premium
                        .max_requests
                        .map_or(false, |max| premium.num_requests >= max)
                } else {
                    standard
                        .max_requests
                        .map_or(false, |max| standard.num_requests >= max)
                }
            })
            .unwrap_or(false);

        // 如果达到限制,直接返回未授权错误
        if need_profile_check {
            state.active_requests -= 1;
            state.error_requests += 1;
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ChatError::Unauthorized.to_json()),
            ));
        }

        let next_id = state.request_logs.last().map_or(1, |log| log.id + 1);
        current_id = next_id;

        // 如果需要获取用户使用情况,创建后台任务获取profile
        if model.map(|m| m.is_usage_check()).unwrap_or(false) {
            let auth_token_clone = auth_token.clone();
            let state_clone = state_clone.clone();
            let log_id = next_id;

            tokio::spawn(async move {
                let profile = get_token_profile(&auth_token_clone).await;
                let mut state = state_clone.lock().await;
                // 根据id查找对应的日志
                if let Some(log) = state
                    .request_logs
                    .iter_mut()
                    .rev()
                    .find(|log| log.id == log_id)
                {
                    log.token_info.profile = profile;
                }
            });
        }

        state.request_logs.push(RequestLog {
            id: next_id,
            timestamp: request_time,
            model: request.model.clone(),
            token_info: TokenInfo {
                token: auth_token.clone(),
                checksum: checksum.clone(),
                profile: None,
            },
            prompt: None,
            timing: TimingInfo {
                total: 0.0,
                first: None,
            },
            stream: request.stream,
            status: STATUS_PENDING,
            error: None,
        });

        if state.request_logs.len() > REQUEST_LOGS_LIMIT {
            state.request_logs.remove(0);
        }
    }

    // 将消息转换为hex格式
    let hex_data = match super::adapter::encode_chat_message(request.messages, &request.model).await
    {
        Ok(data) => data,
        Err(e) => {
            let mut state = state.lock().await;
            if let Some(log) = state
                .request_logs
                .iter_mut()
                .rev()
                .find(|log| log.id == current_id)
            {
                log.status = STATUS_FAILED;
                log.error = Some(e.to_string());
            }
            state.active_requests -= 1;
            state.error_requests += 1;
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    ChatError::RequestFailed("Failed to encode chat message".to_string()).to_json(),
                ),
            ));
        }
    };

    // 构建请求客户端
    // let client_key = match generate_client_key(&checksum) {
    //     Some(key) => key,
    //     None => {
    //         let mut state = state.lock().await;
    //         if let Some(log) = state
    //             .request_logs
    //             .iter_mut()
    //             .rev()
    //             .find(|log| log.id == current_id)
    //         {
    //             log.status = STATUS_FAILED;
    //             log.error = Some(ERR_CHECKSUM_NO_GOOD.to_string());
    //         }
    //         state.active_requests -= 1;
    //         state.error_requests += 1;
    //         return Err((
    //             StatusCode::BAD_REQUEST,
    //             Json(ChatError::RequestFailed(ERR_CHECKSUM_NO_GOOD.to_string()).to_json()),
    //         ));
    //     }
    // };
    let client = build_client(&auth_token, &checksum);
    let response = client.body(hex_data).send().await;

    // 处理请求结果
    let response = match response {
        Ok(resp) => {
            // 更新请求日志为成功
            {
                let mut state = state.lock().await;
                if let Some(log) = state
                    .request_logs
                    .iter_mut()
                    .rev()
                    .find(|log| log.id == current_id)
                {
                    log.status = STATUS_SUCCESS;
                }
            }
            resp
        }
        Err(e) => {
            // 更新请求日志为失败
            {
                let mut state = state.lock().await;
                if let Some(log) = state
                    .request_logs
                    .iter_mut()
                    .rev()
                    .find(|log| log.id == current_id)
                {
                    log.status = STATUS_FAILED;
                    log.error = Some(e.to_string());
                }
                state.active_requests -= 1;
                state.error_requests += 1;
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatError::RequestFailed(e.to_string()).to_json()),
            ));
        }
    };

    // 释放活动请求计数
    {
        let mut state = state.lock().await;
        state.active_requests -= 1;
    }

    if request.stream {
        let response_id = format!("chatcmpl-{}", Uuid::new_v4().simple());
        let full_text = Arc::new(Mutex::new(String::with_capacity(1024)));
        let is_start = Arc::new(AtomicBool::new(true));
        let start_time = std::time::Instant::now();
        let first_chunk_time = Arc::new(Mutex::new(None));

        let stream = {
            // 创建新的 stream
            let mut stream = response.bytes_stream();

            let enable_stream_check = AppConfig::get_stream_check();

            if enable_stream_check {
                // 检查第一个 chunk
                match stream.next().await {
                    Some(first_chunk) => {
                        let chunk = first_chunk.map_err(|e| {
                            let error_message = format!("Failed to read response chunk: {}", e);
                            // 理论上，若程序正常，必定成功，因为前面判断过了
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ChatError::RequestFailed(error_message).to_json()),
                            )
                        })?;

                        match parse_stream_data(&chunk) {
                            Err(StreamError::ChatError(error)) => {
                                let error_respone = error.to_error_response();
                                // 更新请求日志为失败
                                {
                                    let mut state = state.lock().await;
                                    if let Some(log) = state
                                        .request_logs
                                        .iter_mut()
                                        .rev()
                                        .find(|log| log.id == current_id)
                                    {
                                        log.status = STATUS_FAILED;
                                        log.error = Some(error_respone.native_code());
                                        log.timing.total = format_time_ms(start_time.elapsed().as_secs_f64());
                                        state.error_requests += 1;
                                    }
                                }
                                return Err((
                                    error_respone.status_code(),
                                    Json(error_respone.to_common()),
                                ));
                            }
                            Ok(_) | Err(_) => {
                                // 创建一个包含第一个 chunk 的 stream
                                Box::pin(
                                    futures::stream::once(async move { Ok(chunk) }).chain(stream),
                                )
                                    as Pin<
                                        Box<
                                            dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send,
                                        >,
                                    >
                            }
                        }
                    }
                    None => {
                        // Box::pin(stream)
                        //     as Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>
                        // 更新请求日志为失败
                        {
                            let mut state = state.lock().await;
                            if let Some(log) = state
                                .request_logs
                                .iter_mut()
                                .rev()
                                .find(|log| log.id == current_id)
                            {
                                log.status = STATUS_FAILED;
                                log.error = Some("Empty stream response".to_string());
                                state.error_requests += 1;
                            }
                        }
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(
                                ChatError::RequestFailed("Empty stream response".to_string())
                                    .to_json(),
                            ),
                        ));
                    }
                }
            } else {
                Box::pin(stream)
                    as Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>
            }
        }
        .then({
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let first_chunk_time = first_chunk_time.clone();
            let state = state.clone();

            move |chunk| {
                let buffer = buffer.clone();
                let response_id = response_id.clone();
                let model = request.model.clone();
                let is_start = is_start.clone();
                let full_text = full_text.clone();
                let first_chunk_time = first_chunk_time.clone();
                let state = state.clone();

                async move {
                    let chunk = chunk.unwrap_or_default();
                    let mut buffer_guard = buffer.lock().await;
                    buffer_guard.extend_from_slice(&chunk);

                    match parse_stream_data(&buffer_guard) {
                        Ok(StreamMessage::Content(texts)) => {
                            buffer_guard.clear();
                            let mut response_data = String::new();

                            // 记录首字时间(如果还未记录)
                            if let Ok(mut first_time) = first_chunk_time.try_lock() {
                                if first_time.is_none() {
                                    *first_time = Some(format_time_ms(start_time.elapsed().as_secs_f64()));
                                }
                            }

                            // 处理文本内容
                            for text in texts {
                                let mut text_guard = full_text.lock().await;
                                text_guard.push_str(&text);
                                let is_first = is_start.load(Ordering::SeqCst);

                                let response = ChatResponse {
                                    id: response_id.clone(),
                                    object: OBJECT_CHAT_COMPLETION_CHUNK.to_string(),
                                    created: chrono::Utc::now().timestamp(),
                                    model: if is_first { Some(model.clone()) } else { None },
                                    choices: vec![Choice {
                                        index: 0,
                                        message: None,
                                        delta: Some(Delta {
                                            role: if is_first {
                                                is_start.store(false, Ordering::SeqCst);
                                                Some(Role::Assistant)
                                            } else {
                                                None
                                            },
                                            content: Some(text),
                                        }),
                                        finish_reason: None,
                                    }],
                                    usage: None,
                                };

                                response_data.push_str(&format!(
                                    "data: {}\n\n",
                                    serde_json::to_string(&response).unwrap()
                                ));
                            }

                            Ok::<_, Infallible>(Bytes::from(response_data))
                        }
                        Ok(StreamMessage::StreamStart) => {
                            buffer_guard.clear();
                            // 发送初始响应，包含模型信息
                            let response = ChatResponse {
                                id: response_id.clone(),
                                object: OBJECT_CHAT_COMPLETION_CHUNK.to_string(),
                                created: chrono::Utc::now().timestamp(),
                                model: {
                                    is_start.store(true, Ordering::SeqCst);
                                    Some(model.clone())
                                },
                                choices: vec![Choice {
                                    index: 0,
                                    message: None,
                                    delta: Some(Delta {
                                        role: Some(Role::Assistant),
                                        content: Some(String::new()),
                                    }),
                                    finish_reason: None,
                                }],
                                usage: None,
                            };

                            Ok(Bytes::from(format!(
                                "data: {}\n\n",
                                serde_json::to_string(&response).unwrap()
                            )))
                        }
                        Ok(StreamMessage::StreamEnd) => {
                            buffer_guard.clear();
                            // 根据配置决定是否发送最后的 finish_reason
                            let include_finish_reason = AppConfig::get_stop_stream();

                            // 计算总时间和首次片段时间
                            let total_time = format_time_ms(start_time.elapsed().as_secs_f64());
                            let first_time = first_chunk_time.lock().await.unwrap_or(total_time);

                            {
                                let mut state = state.lock().await;
                                if let Some(log) = state
                                    .request_logs
                                    .iter_mut()
                                    .rev()
                                    .find(|log| log.id == current_id)
                                {
                                    log.timing.total = total_time;
                                    log.timing.first = Some(first_time);
                                }
                            }

                            if include_finish_reason {
                                let response = ChatResponse {
                                    id: response_id.clone(),
                                    object: OBJECT_CHAT_COMPLETION_CHUNK.to_string(),
                                    created: chrono::Utc::now().timestamp(),
                                    model: None,
                                    choices: vec![Choice {
                                        index: 0,
                                        message: None,
                                        delta: Some(Delta {
                                            role: None,
                                            content: None,
                                        }),
                                        finish_reason: Some(FINISH_REASON_STOP.to_string()),
                                    }],
                                    usage: None,
                                };
                                Ok(Bytes::from(format!(
                                    "data: {}\n\ndata: [DONE]\n\n",
                                    serde_json::to_string(&response).unwrap()
                                )))
                            } else {
                                Ok(Bytes::from("data: [DONE]\n\n"))
                            }
                        }
                        Ok(StreamMessage::Incomplete) => {
                            // 保持buffer中的数据以待下一个chunk
                            Ok(Bytes::new())
                        }
                        Ok(StreamMessage::Debug(debug_prompt)) => {
                            buffer_guard.clear();
                            if let Ok(mut state) = state.try_lock() {
                                if let Some(last_log) = state.request_logs.last_mut() {
                                    last_log.prompt = Some(debug_prompt.clone());
                                }
                            }
                            Ok(Bytes::new())
                        }
                        Err(e) => {
                            buffer_guard.clear();
                            eprintln!("[警告] Stream error: {}", e);
                            Ok(Bytes::new())
                        }
                    }
                }
            }
        });

        Ok(Response::builder()
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header(CONTENT_TYPE, "text/event-stream")
            .body(Body::from_stream(stream))
            .unwrap())
    } else {
        // 非流式响应
        let start_time = std::time::Instant::now();
        let mut first_chunk_received = false;
        let mut first_chunk_time = 0.0;
        let mut full_text = String::with_capacity(1024);
        let mut stream = response.bytes_stream();
        let mut prompt = None;

        let mut buffer = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                // 更新请求日志为失败
                if let Ok(mut state) = state.try_lock() {
                    if let Some(log) = state
                        .request_logs
                        .iter_mut()
                        .rev()
                        .find(|log| log.id == current_id)
                    {
                        log.status = STATUS_FAILED;
                        log.error = Some(format!("Failed to read response chunk: {}", e));
                        state.error_requests += 1;
                    }
                }
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        ChatError::RequestFailed(format!("Failed to read response chunk: {}", e))
                            .to_json(),
                    ),
                )
            })?;

            buffer.extend_from_slice(&chunk);

            match parse_stream_data(&buffer) {
                Ok(StreamMessage::Content(texts)) => {
                    if !first_chunk_received {
                        first_chunk_time = format_time_ms(start_time.elapsed().as_secs_f64());
                        first_chunk_received = true;
                    }
                    for text in texts {
                        full_text.push_str(&text);
                    }
                    buffer.clear();
                }
                Ok(StreamMessage::Incomplete) => continue,
                Ok(StreamMessage::Debug(debug_prompt)) => {
                    prompt = Some(debug_prompt);
                    buffer.clear();
                }
                Ok(StreamMessage::StreamStart) | Ok(StreamMessage::StreamEnd) => {
                    buffer.clear();
                }
                Err(StreamError::ChatError(error)) => {
                    let error = error.to_error_response();
                    // 更新请求日志为失败
                    {
                        let mut state = state.lock().await;
                        if let Some(log) = state
                            .request_logs
                            .iter_mut()
                            .rev()
                            .find(|log| log.id == current_id)
                        {
                            log.status = STATUS_FAILED;
                            log.error = Some(error.native_code());
                            log.timing.total = format_time_ms(start_time.elapsed().as_secs_f64());
                            state.error_requests += 1;
                        }
                    }
                    return Err((error.status_code(), Json(error.to_common())));
                }
                Err(_) => {
                    buffer.clear();
                    continue;
                }
            }
        }

        // 检查响应是否为空
        if full_text.is_empty() {
            // 更新请求日志为失败
            {
                let mut state = state.lock().await;
                if let Some(log) = state
                    .request_logs
                    .iter_mut()
                    .rev()
                    .find(|log| log.id == current_id)
                {
                    log.status = STATUS_FAILED;
                    log.error = Some("Empty response received".to_string());
                    if let Some(p) = prompt {
                        log.prompt = Some(p);
                    }
                    state.error_requests += 1;
                }
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatError::RequestFailed("Empty response received".to_string()).to_json()),
            ));
        }

        let response_data = ChatResponse {
            id: format!("chatcmpl-{}", Uuid::new_v4().simple()),
            object: OBJECT_CHAT_COMPLETION.to_string(),
            created: chrono::Utc::now().timestamp(),
            model: Some(request.model),
            choices: vec![Choice {
                index: 0,
                message: Some(Message {
                    role: Role::Assistant,
                    content: MessageContent::Text(full_text),
                }),
                delta: None,
                finish_reason: Some(FINISH_REASON_STOP.to_string()),
            }],
            usage: Some(Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }),
        };

        {
            // 更新请求日志时间信息和状态
            let total_time = format_time_ms(start_time.elapsed().as_secs_f64());
            let mut state = state.lock().await;
            if let Some(log) = state
                .request_logs
                .iter_mut()
                .rev()
                .find(|log| log.id == current_id)
            {
                log.timing.total = total_time;
                log.timing.first = Some(first_chunk_time);
                log.prompt = prompt;
                log.status = STATUS_SUCCESS;
            }
        }

        Ok(Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_string(&response_data).unwrap()))
            .unwrap())
    }
}
