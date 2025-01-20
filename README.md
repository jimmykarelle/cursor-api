# cursor-api

## 获取key

1. 访问 [www.cursor.com](https://www.cursor.com) 并完成注册登录
2. 在浏览器中打开开发者工具（F12）
3. 在 Application-Cookies 中查找名为 `WorkosCursorSessionToken` 的条目，并复制其第三个字段。请注意，%3A%3A 是 :: 的 URL 编码形式，cookie 的值使用冒号 (:) 进行分隔。

## 接口说明

### 基础对话（请求格式和响应格式参考 openai）

- 接口地址：`/v1/chat/completions`
- 请求方法：POST
- 认证方式：Bearer Token
  1. 使用环境变量 `AUTH_TOKEN` 进行认证
  2. 使用 `.token` 文件中的令牌列表进行轮询认证

### Token管理接口

#### 简易Token信息管理页面
- 接口地址：`/tokeninfo`
- 请求方法：GET
- 响应格式：HTML页面
- 功能：获取 .token 和 .token-list 文件内容，并允许用户方便地使用 API 修改文件内容

#### 更新Token信息
- 接口地址：`/update-tokeninfo`
- 请求方法：GET
- 认证方式：不需要
- 功能：请求内容不包括文件内容，直接修改文件，调用重载函数

#### 更新Token信息
- 接口地址：`/update-tokeninfo`
- 请求方法：POST
- 认证方式：Bearer Token
- 功能：请求内容包括文件内容，间接修改文件，调用重载函数

#### 获取Token信息
- 接口地址：`/get-tokeninfo`
- 请求方法：POST
- 认证方式：Bearer Token

### 其他接口

#### 获取模型列表
- 接口地址：`/v1/models`
- 请求方法：GET

#### 获取随机x-cursor-checksum
- 接口地址：`/checksum`
- 请求方法：GET

#### 健康检查接口
- 接口地址：`/`
- 请求方法：GET

#### 获取日志接口
- 接口地址：`/logs`
- 请求方法：GET

## 配置说明

### 环境变量

- `PORT`: 服务器端口号（默认：3000）
- `AUTH_TOKEN`: 认证令牌（必须，用于API认证）
- `ROUTE_PREFIX`: 路由前缀（可选）
- `TOKEN_FILE`: token文件路径（默认：.token）
- `TOKEN_LIST_FILE`: token列表文件路径（默认：.token-list）

### Token文件格式

1. `.token` 文件：每行一个token，支持以下格式：

   ```
   # 这是注释
   token1
   # alias与标签的作用差不多
   alias::token2
   ```

   alias 可以是任意值，用于区分不同的 token，更方便管理，WorkosCursorSessionToken 是相同格式
   该文件将自动向.token-list文件中追加token，同时自动生成checksum

2. `.token-list` 文件：每行为token和checksum的对应关系：

   ```
   # 这里的#表示这行在下次读取要删除
   token1,checksum1
   # 支持像.token一样的alias，冲突时以.token为准
   alias::token2,checksum2
   ```

   该文件可以被自动管理，但用户仅可在确认自己拥有修改能力时修改，一般仅有以下情况需要手动修改：

   - 需要删除某个 token
   - 需要使用已有 checksum 来对应某一个 token

### 模型列表

写死了，后续也不会会支持自定义模型列表
```
claude-3.5-sonnet
gpt-4
gpt-4o
claude-3-opus
cursor-fast
cursor-small
gpt-3.5-turbo
gpt-4-turbo-2024-04-09
gpt-4o-128k
gemini-1.5-flash-500k
claude-3-haiku-200k
claude-3-5-sonnet-200k
claude-3-5-sonnet-20241022
gpt-4o-mini
o1-mini
o1-preview
o1
claude-3.5-haiku
gemini-exp-1206
gemini-2.0-flash-thinking-exp
gemini-2.0-flash-exp
```

## 部署

### Docker 部署

#### Docker 运行示例

```bash
docker run -d -e PORT=3000 -e AUTH_TOKEN=your_token -p 3000:3000 wisdgod/cursor-api:latest
```

### huggingface部署

前提：一个huggingface账号

1. 创建一个Space并创建一个Dockerfile文件，内容如下：

   ```Dockerfile
   FROM wisdgod/cursor-api:latest

   # 可能你要覆盖原镜像的环境变量，但都可以在下面的第2步中配置
   ENV PORT=7860
   ```

2. 配置环境变量

   在你的 Space 中，点击 Settings，找到 `Variables and secrets`，添加 Variables

   ```env
   # 可选，用于配置服务器端口
   PORT=3000
   # 必选，用于配置路由前缀，比如/api,/hf,/proxy等等
   ROUTE_PREFIX=
   # 必选，用于API认证
   AUTH_TOKEN=
   # 可选，用于配置token文件路径
   TOKEN_FILE=.token
   # 可选，用于配置token列表文件路径
   TOKEN_LIST_FILE=.token-list
   ```

   更多变量示例可访问 /env-example

3. 重新部署

   点击`Factory rebuild`，等待部署完成

4. 接口地址（`Embed this Space`中查看）：

   ```
   https://{username}-{space-name}.hf.space/v1/models
   ```

## 注意事项

1. 请妥善保管您的任何 Token，不要泄露给他人。若发现泄露，请及时更改
2. 请遵守本项目许可证，你仅拥有使用本项目的权利，不得用于商业用途
3. 本项目仅供学习研究使用，请遵守 Cursor 的使用条款

## 开发

### 跨平台编译

自行配置cross编译环境

支持的平台：

- linux x86_64
- windows x86_64
- macos x86_64
- freebsd x86_64
- docker (only for linux x86_64)

### 获取token

- 使用 [get-token](https://github.com/wisdgod/cursor-api/tree/main/get-token) 获取读取当前设备token，仅支持windows与macos

## 鸣谢

- [cursor-api](https://github.com/wisdgod/cursor-api)
- [zhx47/cursor-api](https://github.com/zhx47/cursor-api)
- [luolazyandlazy/cursorToApi](https://github.com/luolazyandlazy/cursorToApi)

## 许可证

版权所有 (c) 2024

本软件仅供学习和研究使用。未经授权，不得用于商业用途。
保留所有权利。